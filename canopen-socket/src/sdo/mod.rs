//! Service Data Object (SDO) types and utilities.

// TODO
#![allow(missing_docs)]

use can_socket::CanFrame;

use crate::CanOpenSocket;

mod address;
pub use address::*;

mod error;
pub use error::*;

mod read;
pub(crate) use read::*;

mod write;
pub(crate) use write::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
enum ClientCommand {
	SegmentDownload = 0,
	InitiateDownload = 1,
	InitiateUpload = 2,
	SegmentUpload = 3,
	AbortTransfer = 4,
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

fn get_response_command(frame: &CanFrame) -> Result<ServerCommand, SdoError> {
	let data = frame.data();
	if data.len() < 8 {
		return Err(MalformedResponse::WrongFrameSize(data.len()).into());
	}

	let command = ServerCommand::try_from(data[0] >> 5)
		.map_err(|e| MalformedResponse::InvalidServerCommand(e.number))?;
	Ok(command)
}

fn check_server_command(frame: &CanFrame, expected: ServerCommand) -> Result<(), SdoError> {
	let command = get_response_command(frame)?;
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

async fn send_abort_transfer_command(
	bus: &mut CanOpenSocket,
	address: SdoAddress,
	node_id: u8,
	object_index: u16,
	object_subindex: u8,
	reason: error::AbortReason,
) -> Result<(), SdoError> {
	let reason = u32::from(reason).to_be_bytes();
	let object_index = object_index.to_be_bytes();
	let data: [u8; 8] = [
		(ClientCommand::AbortTransfer as u8) << 5,
		object_index[0],
		object_index[1],
		object_subindex,
		reason[0],
		reason[1],
		reason[2],
		reason[3],
	];
	let command = CanFrame::new(address.command_id(node_id), &data, None).unwrap();
	bus.socket.send(&command).await
		.map_err(SdoError::SendFailed)
}
