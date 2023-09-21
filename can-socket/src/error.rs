#[derive(Debug, Clone)]
pub struct InvalidId {
	pub(crate) id: Option<u32>,
	pub(crate) extended: bool,
}

#[derive(Debug, Clone)]
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

impl std::error::Error for InvalidId {}
impl std::fmt::Display for InvalidId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match (self.id, self.extended) {
			(Some(id), false) => write!(f, "invalid base (non-extened) CAN ID: 0x{:03X}, maximum valid value is 0x7FF", id),
			(None, false) => write!(f, "invalid base (non-extened) CAN ID: allowed values are 0 to 0x7FF"),
			(Some(id), true) => write!(f, "invalid extened CAN ID: 0x{:08X}, maximum valid value is 0x1FFF_FFFF", id),
			(None, true) => write!(f, "invalid extened CAN ID, allowed values are 0 to 0x1FFF_FFFF"),
		}
	}
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
