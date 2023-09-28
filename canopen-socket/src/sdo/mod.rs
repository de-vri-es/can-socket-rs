//! Service Data Object (SDO) types and utilities.

use can_socket::CanFrame;

use crate::CanOpenSocket;

mod address;
pub use address::*;

mod error;
pub use error::*;

mod upload;
pub(crate) use upload::*;

mod download;
pub(crate) use download::*;

/// SDO command that can be sent by a client.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
enum ClientCommand {
	/// Download a segment to the server.
	SegmentDownload = 0,

	/// Initiate a download to the server.
	InitiateDownload = 1,

	/// Initiate an upload from the server.
	InitiateUpload = 2,

	/// Request the server to upload a segment.
	SegmentUpload = 3,

	/// Tell the server we are aborting the transfer.
	AbortTransfer = 4,
}

/// SDO command that can be sent by a server.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
enum ServerCommand {
	/// The server is uploading a segment.
	SegmentUpload = 0,

	/// The server has downloaded the segment.
	SegmentDownload = 1,

	/// The server accepts the upload request.
	InitiateUpload = 2,

	/// The server accepts the download request.
	InitiateDownload = 3,

	/// The server is aborting the transfer.
	AbortTransfer = 4,
}

/// Extract the response command from a CAN frame.
///
/// The CAN frame should be an SDO response from an SDO server.
fn get_server_command(frame: &CanFrame) -> Result<ServerCommand, SdoError> {
	let data = frame.data();
	if data.len() < 8 {
		return Err(MalformedResponse::WrongFrameSize(data.len()).into());
	}

	let command = ServerCommand::try_from(data[0] >> 5)
		.map_err(|e| MalformedResponse::InvalidServerCommand(e.number))?;
	Ok(command)
}

/// Check if the response command is the expected one.
///
/// Has special handling for [`ServerCommand::AbortTransfer`] to return a [`TransferAborted`] error.
fn check_server_command(frame: &CanFrame, expected: ServerCommand) -> Result<(), SdoError> {
	let command = get_server_command(frame)?;
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

/// Send an abort command to an SDO server.
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

impl std::fmt::Display for ClientCommand {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ClientCommand::SegmentDownload => write!(f, "download-segment"),
			ClientCommand::InitiateDownload => write!(f, "initiate-download"),
			ClientCommand::InitiateUpload => write!(f, "initiate-upload"),
			ClientCommand::SegmentUpload => write!(f, "upload-segment"),
			ClientCommand::AbortTransfer => write!(f, "abort-transfer"),
		}
	}
}

impl std::fmt::Display for ServerCommand {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ServerCommand::SegmentDownload => write!(f, "download-segment"),
			ServerCommand::InitiateDownload => write!(f, "initiate-download"),
			ServerCommand::InitiateUpload => write!(f, "initiate-upload"),
			ServerCommand::SegmentUpload => write!(f, "upload-segment"),
			ServerCommand::AbortTransfer => write!(f, "abort-transfer"),
		}
	}
}
