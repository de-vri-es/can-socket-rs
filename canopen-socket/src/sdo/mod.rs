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
pub(crate) use read::*;

mod write;
pub(crate) use write::*;

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
