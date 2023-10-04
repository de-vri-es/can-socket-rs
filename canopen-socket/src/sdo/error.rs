/// Error that can occur during a typed SDO upload.
#[derive(Debug)]
#[derive(thiserror::Error)]
#[error("{0}")]
pub enum UploadError<E> {
	/// An error occured during the SDO transfer.
	UploadFailed(SdoError),

	/// The parsing of the received data failed.
	ParseFailed(E),
}

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

	/// The buffer is to small to receive the requested object.
	BufferTooSmall(#[from] BufferTooSmall),

	/// The transfer was aborted by the SDO server.
	TransferAborted(#[from] TransferAborted),

	/// The response from the server does not follow the correct format for an SDO response.
	MalformedResponse(#[from] MalformedResponse),

	/// Received an SDO response with an unexpected server command.
	UnexpectedResponse(#[from] UnexpectedResponse),

	/// The flags on the message are not valid.
	#[error("Invalid flags in server response: neither the expedited nor the size flags is set")]
	NoExpeditedOrSizeFlag,

	/// The toggle flag is not in the expected state.
	#[error("Invalid toggle flag in server response")]
	InvalidToggleFlag,

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

/// The buffer is too small to receive the requested object.
#[derive(Debug)]
#[derive(thiserror::Error)]
#[error("Buffer is too small to receive the requested data, buffer size is {available} bytes, need atleast {needed}")]
pub struct BufferTooSmall {
	/// The buffer size.
	pub(super) available: usize,

	/// The minimum buffer size needed to receive the object.
	pub(super) needed: usize,
}

/// The transfer was aborted by the SDO server.
#[derive(Debug)]
#[derive(thiserror::Error)]
pub struct TransferAborted {
	/// The reason from the server for aborting the transfer.
	pub(super) reason: Result<super::AbortReason, u32>,
}

impl std::fmt::Display for TransferAborted {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match  &self.reason {
			Ok(reason) => write!(f, "SDO transfer aborted by server: {reason}"),
			Err(unknown_reason) => write!(f, "SDO transfer aborted by server with unknown reason code: 0x{unknown_reason:04X}"),
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
