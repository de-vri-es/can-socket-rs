#[derive(Debug)]
pub enum SdoError {
	InvalidCanId(can_socket::error::InvalidId),
	DataLengthExceedsMaximum(DataLengthExceedsMaximum),
	SendFailed(std::io::Error),
	RecvFailed(std::io::Error),
	Timeout,
	TransferAborted(TransferAborted),
	MalformedResponse(MalformedResponse),
	UnexpectedResponse(UnexpectedResponse),
	WrongDataCount(WrongDataCount),
}

#[derive(Debug)]
pub struct DataLengthExceedsMaximum {
	pub(super) data_len: usize,
}

#[derive(Debug)]
pub struct TransferAborted {
	pub(super) reason: Result<AbortReason, u32>,
}

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

#[derive(Debug)]
pub enum MalformedResponse {
	WrongFrameSize(usize),
	InvalidServerCommand(u8),
	InvalidFlags,
	InvalidToggleFlag,
	TooManySegments,
}

#[derive(Debug)]
pub struct UnexpectedResponse {
	pub(super) expected: super::ServerCommand,
	pub(super) actual: super::ServerCommand,
}

#[derive(Debug)]
pub struct WrongDataCount {
	pub(super) expected: usize,
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
