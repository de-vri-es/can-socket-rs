//! Service Data Object (SDO) types and utilities.

use can_socket::CanFrame;

use crate::{CanOpenSocket, ObjectIndex};

mod address;
pub use address::*;

mod error;
pub use error::*;

mod upload;
pub use upload::*;

mod download;
pub use download::*;

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

/// The reason for aborting a transfer.
///
/// Definitions come from CiA 301 section 7.2.3.3.17 table 22.
#[derive(Debug)]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u32)]
pub enum AbortReason {
	/// Toggle bit not alternated.
	ToggleBitNotAlternated = 0x0503_0000,

	/// SDO protocol timed out.
	SdoProtocolTimedOut = 0x0504_0000,

	/// Client/server command specifier not valid or unknown.
	InvalidOrUnknownCommandSpecifier = 0x0504_0001,

	/// Invalid block size (block mode only).
	InvalidBlockSize = 0x0504_0002,

	/// Invalid sequence number (block mode only).
	InvalidSequenceNumber = 0x0504_0003,

	/// CRC error (block mode only).
	CrcError = 0x0504_0004,

	/// Out of memory.
	OutOfMemory = 0x0504_0005,

	/// Unsupported access to an object.
	UnsupportedObjectAccess = 0x0601_0000,

	/// Attempt to read a write only object.
	ReadFromWriteOnlyObject = 0x0601_0001,

	/// Attempt to write a read only object.
	WriteToReadOnlyObject = 0x0601_0002,

	/// Object does not exist in the object dictionary.
	ObjectDoesNotExist = 0x0602_0000,

	/// Object cannot be mapped to the PDO.
	ObjectCanNotBeMapped = 0x0604_0041,

	/// The number and length of the objects to be mapped would exceed PDO length.
	NumberAndLengthOfObjectsExceedPdoLength = 0x0604_0042,

	/// General parameter incompatibility reason.
	GeneralParameterError = 0x0604_0043,

	/// General internal incompatibility in the device.
	GeneralInternalError = 0x0604_0047,

	/// Access failed due to an hardware error.
	HardwareError = 0x0606_0000,

	/// Data type does not match, length of service parameter does not match
	LengthMismatch = 0x0607_0010,

	/// Data type does not match, length of service parameter too high
	LengthTooHigh = 0x0607_0012,

	/// Data type does not match, length of service parameter too low
	LengthTooLow = 0x0607_0013,

	/// Sub-index does not exist.
	SubIndexDoesNotExist = 0x0609_0011,

	/// Invalid value for parameter (download only).
	ObjectValueInvalid = 0x0609_0030,

	/// Value of parameter written too high (download only).
	ObjectValueTooHigh = 0x0609_0031,

	/// Value of parameter written too low (download only).
	ObjectValueTooLow = 0x0609_0032,

	/// Maximum value is less than minimum value.
	MaximumBelowMinimum = 0x0609_0036,

	/// Resource not available: SDO connection
	ResourceNotAvailable = 0x060A_0023,

	/// General error
	GeneralError = 0x0800_0000,

	/// Data cannot be transferred or stored to the application.
	CanNotTransferData = 0x0800_0020,

	/// Data cannot be transferred or stored to the application because of local control.
	LocalControlError = 0x0800_0021,

	/// Data cannot be transferred or stored to the application because of the present device state.
	InvalidDeviceStateForTransfer = 0x0800_0022,

	/// Object dictionary dynamic generation fails or no object dictionary is present (e.g. object dictionary is generated from file and generation fails because of an file error).
	FailedToGenerateDynamicDictionary = 0x0800_0023,

	/// No data available
	NoDataAvailable = 0x0800_0024,
}

/// Extract the response command from a CAN frame.
///
/// The CAN frame should be an SDO response from an SDO server.
fn get_server_command(frame: &CanFrame) -> Result<(ServerCommand, [u8; 8]), SdoError> {
	let data = frame.data()
		.ok_or(SdoError::MalformedResponse(MalformedResponse::WrongFrameSize(0)))?;
	let data: [u8; 8] = data.as_slice()
		.try_into()
		.map_err(|_| MalformedResponse::WrongFrameSize(data.len()))?;

	let command = ServerCommand::try_from(data[0] >> 5)
		.map_err(|e| MalformedResponse::InvalidServerCommand(e.number))?;
	Ok((command, data))
}

/// Check if the response command is the expected one.
///
/// Has special handling for [`ServerCommand::AbortTransfer`] to return a [`TransferAborted`] error.
fn check_server_command(frame: &CanFrame, expected: ServerCommand) -> Result<[u8; 8], SdoError> {
	let (command, data) = get_server_command(frame)?;
	if command == expected {
		Ok(data)
	} else if command == ServerCommand::AbortTransfer {
		let reason = u32::from_le_bytes(data[4..8].try_into().unwrap());
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
	object: ObjectIndex,
	reason: AbortReason,
) -> Result<(), SdoError> {
	let reason = u32::from(reason).to_be_bytes();
	let object_index = object.index.to_be_bytes();
	let data: [u8; 8] = [
		(ClientCommand::AbortTransfer as u8) << 5,
		object_index[0],
		object_index[1],
		object.subindex,
		reason[0],
		reason[1],
		reason[2],
		reason[3],
	];
	let command = CanFrame::new(address.command_id(node_id), data);
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

impl std::fmt::Display for AbortReason {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ToggleBitNotAlternated => write!(f, "toggle bit not alternated"),
			Self::SdoProtocolTimedOut => write!(f, "SDO protocol timed out"),
			Self::InvalidOrUnknownCommandSpecifier => write!(f, "invalid or unknown SDO command"),
			Self::InvalidBlockSize => write!(f, "invalid block size "),
			Self::InvalidSequenceNumber => write!(f, "invalid sequence number"),
			Self::CrcError => write!(f, "CRC error"),
			Self::OutOfMemory => write!(f, "out of memory"),
			Self::UnsupportedObjectAccess => write!(f, "unsupported access to an object"),
			Self::ReadFromWriteOnlyObject => write!(f, "attempt to read a write only object"),
			Self::WriteToReadOnlyObject => write!(f, "attempt to write a read only object"),
			Self::ObjectDoesNotExist => write!(f, "object does not exist in the object dictionary"),
			Self::ObjectCanNotBeMapped => write!(f, "object cannot be mapped to the PDO"),
			Self::NumberAndLengthOfObjectsExceedPdoLength => write!(f, "the number and length of the objects to be mapped would exceed PDO length"),
			Self::GeneralParameterError => write!(f, "general parameter incompatibility reason"),
			Self::GeneralInternalError => write!(f, "general internal incompatibility in the device"),
			Self::HardwareError => write!(f, "access failed due to an hardware error"),
			Self::LengthMismatch => write!(f, "data type does not match, length of service parameter does not match"),
			Self::LengthTooHigh => write!(f, "data type does not match, length of service parameter too high"),
			Self::LengthTooLow => write!(f, "data type does not match, length of service parameter too low"),
			Self::SubIndexDoesNotExist => write!(f, "sub-index does not exist"),
			Self::ObjectValueInvalid => write!(f, "invalid value for parameter"),
			Self::ObjectValueTooHigh => write!(f, "value of parameter written is too high"),
			Self::ObjectValueTooLow => write!(f, "value of parameter written is too low"),
			Self::MaximumBelowMinimum => write!(f, "maximum value is less than minimum value"),
			Self::ResourceNotAvailable => write!(f, "resource not available: SDO connection"),
			Self::GeneralError => write!(f, "general error"),
			Self::CanNotTransferData => write!(f, "data cannot be transferred or stored to the application"),
			Self::LocalControlError => write!(f, "data cannot be transferred or stored to the application because of local control"),
			Self::InvalidDeviceStateForTransfer => write!(f, "data cannot be transferred or stored to the application because of the present device state"),
			Self::FailedToGenerateDynamicDictionary => write!(f, "dynamic object dictionary generation failed or no object dictionary is present"),
			Self::NoDataAvailable => write!(f, "no data available"),
		}
	}
}
