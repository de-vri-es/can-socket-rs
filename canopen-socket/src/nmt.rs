//! Network Management (NMT) types and utilities.

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::time::Duration;

use can_socket::{CanFrame, CanBaseId};
use crate::CanOpenSocket;

const NMT_COB_ID: u8 = 0x000;

const FUNCTION_HEARTBEAT: u16 = 0x700;

fn heartbeat_id(node_id: u8) -> CanBaseId {
	CanBaseId::new(FUNCTION_HEARTBEAT | u16::from(node_id)).unwrap()
}


/// The NMT state of a CANopen device.
#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NmtState {
	/// The device is initializing and should automatically continue to `PreOperational`.
	Initializing = 0x00,

	/// The device is stopped.
	Stopped = 0x04,

	/// The device is operational.
	Operational = 0x05,

	/// The device has finished initialization and is waiting for a [`NmtCommand::Start`] command.
	PreOperational = 0x7F,
}

/// An NMT command.
#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NmtCommand {
	/// Command a CANopen device to go the [`NmtState::Operational`] state.
	Start = 1,

	/// Command a CANopen device to go the [`NmtState::Stopped`] state.
	Stop = 2,

	/// Command a CANopen device to go the [`NmtState::PreOperational`] state.
	GoToPreOperational = 128,

	/// Command a CANopen device to go the [`NmtState::Initializing`] state.
	Reset = 129,

	/// Command a CANopen device to go the [`NmtState::Initializing`] state but only reset communication parameters.
	ResetCommunication = 130,
}

impl NmtCommand {
	/// Get the expected state for the command.
	fn expected_state(self) -> NmtState {
		match self {
			NmtCommand::Start => NmtState::Operational,
			NmtCommand::Stop => NmtState::Stopped,
			NmtCommand::GoToPreOperational => NmtState::PreOperational,
			NmtCommand::Reset => NmtState::Initializing,
			NmtCommand::ResetCommunication => NmtState::Initializing,
		}
	}
}

/// An error that can occur when sending an NMT command.
#[derive(Debug)]
pub enum NmtError {
	/// Failed to transmit the CAN frame.
	SendFailed(std::io::Error),

	/// Failed to receive a CAN frame for the response.
	RecvFailed(std::io::Error),

	/// The timeout elapsed before the device reported the new state.
	Timeout,

	/// The response frame from the device contains invalid data.
	MalformedResponse,

	/// The new state of the device does not match the expected state.
	UnexpectedState(UnexpectedState)
}

/// The new state of the device does not match the expected state.
#[derive(Debug)]
pub struct UnexpectedState {
	/// The expected state of the device.
	pub expected: NmtState,

	/// The actual state of the device.
	pub actual: NmtState,
}

impl CanOpenSocket {
	/// Send an NMT command and wait for the device to go into the specified state.
	pub async fn send_nmt_command(&mut self, node_id: u8, command: NmtCommand, timeout: Duration) -> Result<(), NmtError> {
		let command_frame = CanFrame::new(
			NMT_COB_ID,
			&[command as u8, node_id],
			None,
		).unwrap();
		self.socket.send(&command_frame)
			.await
			.map_err(NmtError::SendFailed)?;

		let expected = command.expected_state();
		let frame = self.recv_new_by_can_id(heartbeat_id(node_id), timeout)
			.await
			.map_err(NmtError::RecvFailed)?
			.ok_or(NmtError::Timeout)?;
		let state = parse_heartbeat(&frame)?;
		if state == command.expected_state() {
			Ok(())
		} else {
			Err(UnexpectedState { expected, actual: state }.into())
		}
	}
}

/// Parse a heartbeat frame.
fn parse_heartbeat(frame: &CanFrame) -> Result<NmtState, NmtError> {
	if frame.data().len() != 1 {
		Err(NmtError::MalformedResponse)
	} else {
		let state = frame.data()[0].try_into()
			.map_err(|_| NmtError::MalformedResponse)?;
		Ok(state)
	}
}

impl std::error::Error for NmtError {}
impl std::fmt::Display for NmtError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::SendFailed(e) => write!(f, "failed to send CAN frame: {e}"),
			Self::RecvFailed(e) => write!(f, "failed to receive CAN frame: {e}"),
			Self::Timeout => write!(f, "timeout while waiting for reply"),
			Self::MalformedResponse => write!(f, "received malformed response frame"),
			Self::UnexpectedState(e)  => write!(f, "{e}"),
		}
	}
}

impl std::error::Error for UnexpectedState {}
impl std::fmt::Display for UnexpectedState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "state change failed: device reports state {} instead of {}", self.actual, self.expected)
	}
}

impl std::fmt::Display for NmtState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Initializing => write!(f, "initializing"),
			Self::Stopped => write!(f, "stopped"),
			Self::Operational => write!(f, "operation"),
			Self::PreOperational => write!(f, "pre-operational"),
		}
	}
}

impl std::fmt::Display for NmtCommand {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Start => write!(f, "initializing"),
			Self::Stop => write!(f, "stopped"),
			Self::GoToPreOperational => write!(f, "go-to-pre-operational"),
			Self::Reset => write!(f, "reset"),
			Self::ResetCommunication => write!(f, "reset-communication"),
		}
	}
}

impl From<UnexpectedState> for NmtError {
	fn from(value: UnexpectedState) -> Self {
		Self::UnexpectedState(value)
	}
}
