/// Error that can occur during an SDO transfer.
#[derive(Debug)]
#[derive(thiserror::Error)]
#[error("{0}")]
pub enum SdoError {
	/// The data length for the transfer exceeds the maximum size.
	DataLengthExceedsMaximum(#[from] DataLengthExceedsMaximum),

	/// Sending a CAN frame failed.
	#[error("Failed to transmit can frame: {0}")]
	SendFailed(std::io::Error),

	/// Receiving a CAN frame failed.
	#[error("Failed to receive can frame: {0}")]
	RecvFailed(std::io::Error),

	/// A timeout occured while waiting for a response message.
	#[error("Timeout while waiting for response")]
	Timeout,

	/// The transfer was aborted by the SDO server.
	TransferAborted(#[from] TransferAborted),

	/// The response from the server does not follow the correct format for an SDO response.
	MalformedResponse(#[from] MalformedResponse),

	/// Received an SDO response with an unexpected server command.
	UnexpectedResponse(#[from] UnexpectedResponse),

	/// Received a different amount of data then advertised by the server.
	WrongDataCount(#[from] WrongDataCount),
}

/// The data length for the transfer exceeds the maximum size.
#[derive(Debug)]
#[derive(thiserror::Error)]
#[error("Data length is too long for an SDO transfer: length is {data_len}, but the maximum is {}", u32::MAX)]
pub struct DataLengthExceedsMaximum {
	/// The length of the data.
	pub(super) data_len: usize,
}

/// The transfer was aborted by the SDO server.
#[derive(Debug)]
#[derive(thiserror::Error)]
pub struct TransferAborted {
	/// The reason from the server for aborting the transfer.
	pub(super) reason: Result<AbortReason, u32>,
}

impl std::fmt::Display for TransferAborted {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match  &self.reason {
			Ok(reason) => write!(f, "SDO transfer aborted by server: {reason}"),
			Err(unknown_reason) => write!(f, "SDO transfer aborted by server with unknown reason code: 0x{unknown_reason:04X}"),
		}
	}
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
			Self::FailedToGenerateDynamicDictionary => write!(f, "object dictionary dynamic generation failed or no object dictionary is present"),
			Self::NoDataAvailable => write!(f, "no data available"),
		}
	}
}

/// The response from the server does not follow the correct format for an SDO response.
#[derive(Debug)]
#[derive(thiserror::Error)]
pub enum MalformedResponse {
	/// The CAN frame does not have the correct length of 8 data bytes.
	#[error("Wrong frame size: expected 8 bytes, got {0}")]
	WrongFrameSize(usize),

	/// The server command is not valid.
	#[error("Invalid server command: 0x{0:02X}")]
	InvalidServerCommand(u8),

	/// The flags on the message are not valid.
	#[error("Invalid flags in server response")]
	InvalidFlags,

	/// The toggle flag is not in the expected state.
	#[error("Invalid toggle flag in server response")]
	InvalidToggleFlag,

	/// The server is giving us more segments than it should.
	#[error("Received too many data segments from server")]
	TooManySegments,
}

/// Received an SDO response with an unexpected server command.
#[derive(Debug)]
#[derive(thiserror::Error)]
#[error("Unexpected response: expected {expected}, got {actual}")]
pub struct UnexpectedResponse {
	/// The expected server command.
	pub(super) expected: super::ServerCommand,

	/// The actual server command.
	pub(super) actual: super::ServerCommand,
}

/// Received a different amount of data then advertised by the server.
#[derive(Debug)]
#[derive(thiserror::Error)]
#[error("Received wrong amount of data from server, expected {expected} bytes, got {actual}")]
pub struct WrongDataCount {
	/// The expected amount of data as originally advertised by the server.
	pub(super) expected: usize,

	/// The actual amount of data received from the server.
	pub(super) actual: usize,
}
