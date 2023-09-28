/// Error that can occur during an SDO transfer.
#[derive(Debug)]
pub enum SdoError {
	/// The SDO address is invalid for use as a CAN ID.
	InvalidCanId(can_socket::error::InvalidId),

	/// The data length for the transfer exceeds the maximum size.
	DataLengthExceedsMaximum(DataLengthExceedsMaximum),

	/// Sending a CAN frame failed.
	SendFailed(std::io::Error),

	/// Receiving a CAN frame failed.
	RecvFailed(std::io::Error),

	/// A timeout occured while waiting for a response message.
	Timeout,

	/// The transfer was aborted by the SDO server.
	TransferAborted(TransferAborted),

	/// The response from the server does not follow the correct format for an SDO response.
	MalformedResponse(MalformedResponse),

	/// Received an SDO response with an unexpected server command.
	UnexpectedResponse(UnexpectedResponse),

	/// Received a different amount of data then advertised by the server.
	WrongDataCount(WrongDataCount),
}

/// The data length for the transfer exceeds the maximum size.
#[derive(Debug)]
pub struct DataLengthExceedsMaximum {
	/// The length of the data.
	pub(super) data_len: usize,
}

/// The transfer was aborted by the SDO server.
#[derive(Debug)]
pub struct TransferAborted {
	/// The reason from the server for aborting the transfer.
	pub(super) reason: Result<AbortReason, u32>,
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

/// The response from the server does not follow the correct format for an SDO response.
#[derive(Debug)]
pub enum MalformedResponse {
	/// The CAN frame does not have the correct length of 8 data bytes.
	WrongFrameSize(usize),

	/// The server command is not valid.
	InvalidServerCommand(u8),

	/// The flags on the message are not valid.
	InvalidFlags,

	/// The toggle flag is not in the expected state.
	InvalidToggleFlag,

	/// The server is giving us more segments than it should.
	TooManySegments,
}

/// Received an SDO response with an unexpected server command.
#[derive(Debug)]
pub struct UnexpectedResponse {
	/// The expected server command.
	pub(super) expected: super::ServerCommand,

	/// The actual server command.
	pub(super) actual: super::ServerCommand,
}

/// Received a different amount of data then advertised by the server.
#[derive(Debug)]
pub struct WrongDataCount {
	/// The expected amount of data as originally advertised by the server.
	pub(super) expected: usize,

	/// The actual amount of data received from the server.
	pub(super) actual: usize,
}

impl From<DataLengthExceedsMaximum> for SdoError {
	fn from(value: DataLengthExceedsMaximum) -> Self {
		Self::DataLengthExceedsMaximum(value)
	}
}

impl From<TransferAborted> for SdoError {
	fn from(value: TransferAborted) -> Self {
		Self::TransferAborted(value)
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
