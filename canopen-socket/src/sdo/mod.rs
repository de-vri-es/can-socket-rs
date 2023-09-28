//! Service Data Object (SDO) types and utilities.

// TODO
#![allow(missing_docs)]

use can_socket::CanFrame;
use std::time::Duration;

use crate::CanOpenSocket;

mod address;
pub use address::*;

mod error;
pub use error::*;

mod read;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum ClientCommand {
	SegmentDownload = 0,
	InitiateDownload = 1,
	InitiateUpload = 2,
	SegmentUpload = 3,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
enum ServerCommand {
	SegmentUpload = 0,
	SegmentDownload = 1,
	InitiateUpload = 2,
	InitiateDownload = 3,
	AbortTransfer = 4,
}

impl CanOpenSocket {
	/// "Download" an SDO to the server.
	///
	/// Note that download means "download to server",
	/// so most people outside of [CiA](https://can-cia.org/) would call this an upload.
	pub async fn write_sdo(
		&mut self,
		address: SdoAddress,
		node_id: u8,
		object_index: u16,
		object_subindex: u8,
		data: &[u8],
		timeout: Duration,
	) -> Result<(), SdoError> {
		// Can write in a single frame.
		if data.len() <= 4 {
			return self.write_sdo_expidited(address, node_id, object_index, object_subindex, data, timeout).await;
		}
		todo!();
	}

	async fn write_sdo_expidited(
		&mut self,
		address: SdoAddress,
		node_id: u8,
		object_index: u16,
		object_subindex: u8,
		data: &[u8],
		timeout: Duration,
	) -> Result<(), SdoError> {
		let data = make_sdo_data(ClientCommand::InitiateDownload, true, true, object_index, object_subindex, data);
		let frame = CanFrame::new(address.command_id(node_id), &data, None).unwrap();

		self.socket.send(&frame).await
			.map_err(SdoError::SendFailed)?;

		let response = self.recv_new_by_can_id(address.response_id(node_id), timeout)
			.await
			.map_err(SdoError::RecvFailed)?
			.ok_or(SdoError::Timeout)?;

		check_server_command(&response, ServerCommand::InitiateDownload)?;
		Ok(())
	}

	pub async fn read_sdo(
		&mut self,
		address: SdoAddress,
		node_id: u8,
		object_index: u16,
		object_subindex: u8,
		timeout: Duration,
	) -> Result<Vec<u8>, SdoError> {
		read::read_sdo(self, address, node_id, object_index, object_subindex, timeout).await
	}
}


#[allow(clippy::get_first, clippy::identity_op)]
fn make_sdo_data(command: ClientCommand, expedited: bool, size_set: bool, object_index: u16, object_subindex: u8, data: &[u8]) -> [u8; 8] {
	debug_assert!(data.len() <= 4);
	let ccs = command as u8;
	let n = data.len() as u8;
	let expedited: u8 = expedited.into();
	let size_set: u8 = size_set.into();
	[
		ccs << 5 | n << 2 | expedited << 1 | size_set,
		(object_index >> 8 & 0xFF) as u8,
		(object_index >> 0 & 0xFF) as u8,
		object_subindex,
		data.get(0).copied().unwrap_or(0),
		data.get(1).copied().unwrap_or(0),
		data.get(2).copied().unwrap_or(0),
		data.get(3).copied().unwrap_or(0),
	]
}

fn parse_sdo_download_confirmation(frame: &CanFrame) -> Result<(), SdoError> {
	check_server_command(frame, ServerCommand::InitiateDownload)
}

fn check_server_command(frame: &CanFrame, expected: ServerCommand) -> Result<(), SdoError> {
	let data = frame.data();
	if data.len() < 8 {
		return Err(MalformedResponse::WrongFrameSize(data.len()).into());
	}

	let command = ServerCommand::try_from(data[0] >> 5)
		.map_err(|e| MalformedResponse::InvalidServerCommand(e.number))?;
	if command == expected {
		Ok(())
	} else if command == ServerCommand::AbortTransfer {
		let reason = u32::from_le_bytes(frame.data()[4..8].try_into().unwrap());
		let reason = AbortReason::try_from(reason).map_err(|e| e.number);
		Err(SdoError::TransferAborted(TransferAborted { reason }))
	} else {
		Err(UnexpectedResponse { expected, actual: command }.into())
	}
}

impl From<MalformedResponse> for SdoError {
	fn from(value: MalformedResponse) -> Self {
		Self::MalformedResponse(value)
	}
}

impl From<UnexpectedResponse> for SdoError {
	fn from(value: UnexpectedResponse) -> Self {
		Self::UnexpectedResponse(value)
	}
}

impl From<WrongDataCount> for SdoError {
	fn from(value: WrongDataCount) -> Self {
		Self::WrongDataCount(value)
	}
}
