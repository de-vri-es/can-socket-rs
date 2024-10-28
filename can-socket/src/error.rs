//! Error types.

/// The ID was invalid.
#[derive(Debug, Clone)]
pub struct InvalidId {
	pub(crate) id: Option<u32>,
	pub(crate) extended: bool,
}

impl std::error::Error for InvalidId {}
impl std::fmt::Display for InvalidId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match (self.id, self.extended) {
			(Some(id), false) => write!(f, "invalid standard CAN ID: 0x{:03X}, maximum valid value is 0x7FF", id),
			(None, false) => write!(f, "invalid standard CAN ID: allowed values are 0 to 0x7FF"),
			(Some(id), true) => write!(f, "invalid extended CAN ID: 0x{:08X}, maximum valid value is 0x1FFF_FFFF", id),
			(None, true) => write!(f, "invalid extended CAN ID, allowed values are 0 to 0x1FFF_FFFF"),
		}
	}
}

/// Failed to parse the ID.
#[derive(Clone)]
pub struct ParseIdError {
	inner: ParseIdErrorInner,
}

impl ParseIdError {
	pub(crate) fn invalid_format(error: std::num::ParseIntError, extended: bool) -> Self {
		let inner = match error.kind() {
			std::num::IntErrorKind::PosOverflow => ParseIdErrorInner::InvalidValue(InvalidId { id: None, extended }),
			std::num::IntErrorKind::NegOverflow => ParseIdErrorInner::InvalidValue(InvalidId { id: None, extended }),
			_ => ParseIdErrorInner::InvalidFormat(error),
		};
		Self { inner }
	}
	pub(crate) fn invalid_value(error: InvalidId) -> Self {
		let inner = ParseIdErrorInner::InvalidValue(error);
		Self { inner }
	}
}

#[derive(Debug, Clone)]
enum ParseIdErrorInner {
	InvalidFormat(<u16 as std::str::FromStr>::Err),
	InvalidValue(InvalidId),
}

impl std::error::Error for ParseIdError {}

impl std::fmt::Display for ParseIdError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.inner {
			ParseIdErrorInner::InvalidFormat(e) => e.fmt(f),
			ParseIdErrorInner::InvalidValue(e) => e.fmt(f),
		}
	}
}

impl std::fmt::Debug for ParseIdError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Debug::fmt(&self.inner, f)
	}
}

/// The data does not fit in a CAN data frame.
#[derive(Clone, Debug)]
pub struct TryIntoCanDataError {
	pub(crate) len: usize,
}

impl std::error::Error for TryIntoCanDataError {}

impl std::fmt::Display for TryIntoCanDataError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "data to large for CAN frame, expected at most 8 bytes, got {}", self.len)
	}
}

impl From<TryIntoCanDataError> for std::io::Error {
	fn from(value: TryIntoCanDataError) -> Self {
		std::io::Error::new(std::io::ErrorKind::InvalidInput, value.to_string())
	}
}

/// The data or ID used to construct a CAN frame was out of bounds.
#[derive(Clone)]
pub struct TryNewCanFrameError {
	inner: TryNewCanFrameErrorInner,
}

#[derive(Clone, Debug)]
enum TryNewCanFrameErrorInner {
	InvalidId(InvalidId),
	InvalidData(TryIntoCanDataError),
}

impl std::error::Error for TryNewCanFrameError { }

impl std::fmt::Display for TryNewCanFrameError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.inner {
			TryNewCanFrameErrorInner::InvalidId(e) => e.fmt(f),
			TryNewCanFrameErrorInner::InvalidData(e) => e.fmt(f),
		}
	}
}

impl std::fmt::Debug for TryNewCanFrameError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Debug::fmt(&self.inner, f)
	}
}

impl From<std::convert::Infallible> for TryNewCanFrameError {
	fn from(_value: std::convert::Infallible) -> Self {
		unreachable!()
	}
}

impl From<InvalidId> for TryNewCanFrameError {
	fn from(value: InvalidId) -> Self {
		Self {
			inner: TryNewCanFrameErrorInner::InvalidId(value),
		}
	}
}

impl From<TryIntoCanDataError> for TryNewCanFrameError {
	fn from(value: TryIntoCanDataError) -> Self {
		Self {
			inner: TryNewCanFrameErrorInner::InvalidData(value),
		}
	}
}

/// The data length code is too large (maximum possible value is 15).
#[derive(Debug, Clone)]
pub struct InvalidDataLengthCode {
	pub(crate) value: u8,
}

impl std::error::Error for InvalidDataLengthCode {}

impl std::fmt::Display for InvalidDataLengthCode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "invalid data length code: {}, maximum allowed value is 15", self.value)
	}
}
