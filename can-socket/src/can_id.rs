use crate::error::{InvalidId, ParseIdError};

pub const MAX_CAN_ID_BASE: u16 = 0x7FF;
pub const MAX_CAN_ID_EXTENDED: u32 = 0x1FFF_FFFF;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub enum CanId {
	Base(CanBaseId),
	Extended(CanExtendedId),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct CanBaseId {
	id: u16,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct CanExtendedId {
	id: u32,
}

impl CanId {
	pub fn new(id: u32) -> Result<Self, InvalidId> {
		if id <= MAX_CAN_ID_BASE.into() {
			let id = id as u16;
			Ok(Self::Base(CanBaseId { id }))
		} else {
			Ok(Self::Extended(CanExtendedId::new(id)?))
		}
	}

	pub fn new_base(id: u16) -> Result<Self, InvalidId> {
		Ok(Self::Base(CanBaseId::new(id)?))
	}

	pub fn new_extended(id: u32) -> Result<Self, InvalidId> {
		Ok(Self::Extended(CanExtendedId::new(id)?))
	}

	pub fn as_u32(self) -> u32 {
		self.to_extended().as_u32()
	}

	pub fn as_base(self) -> Option<CanBaseId> {
		match self {
			Self::Base(id) => Some(id),
			Self::Extended(_) => None,
		}
	}

	pub fn as_extended(self) -> Option<CanExtendedId> {
		match self {
			Self::Base(_) => None,
			Self::Extended(id) => Some(id),
		}
	}

	pub fn to_extended(self) -> CanExtendedId {
		match self {
			Self::Base(id) => id.into(),
			Self::Extended(id) => id,
		}
	}
}

impl CanBaseId {
	pub fn new(id: u16) -> Result<Self, InvalidId> {
		if id <= MAX_CAN_ID_BASE {
			Ok(Self { id })
		} else {
			Err(InvalidId {
				id: Some(id.into()),
				extended: false,
			})
		}
	}

	pub fn as_u16(self) -> u16 {
		self.id
	}
}

impl CanExtendedId {
	pub fn new(id: u32) -> Result<Self, InvalidId> {
		if id <= MAX_CAN_ID_EXTENDED {
			Ok(Self { id })
		} else {
			Err(InvalidId {
				id: Some(id),
				extended: false,
			})
		}
	}

	pub fn as_u32(self) -> u32 {
		self.id
	}
}

impl PartialEq<CanBaseId> for CanId {
	fn eq(&self, other: &CanBaseId) -> bool {
		self.as_base()
			.map(|x| x == *other)
			.unwrap_or(false)
	}
}

impl PartialEq<CanId> for CanBaseId {
	fn eq(&self, other: &CanId) -> bool {
		other == self
	}
}

impl PartialEq<CanExtendedId> for CanId {
	fn eq(&self, other: &CanExtendedId) -> bool {
		self.as_extended()
			.map(|x| x == *other)
			.unwrap_or(false)
	}
}

impl PartialEq<CanId> for CanExtendedId {
	fn eq(&self, other: &CanId) -> bool {
		other == self
	}
}

impl From<u8> for CanBaseId {
	fn from(value: u8) -> Self {
		Self { id: value.into() }
	}
}

impl TryFrom<u16> for CanBaseId {
	type Error = InvalidId;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<u8> for CanExtendedId {
	fn from(value: u8) -> Self {
		Self { id: value.into() }
	}
}

impl From<u16> for CanExtendedId {
	fn from(value: u16) -> Self {
		Self { id: value.into() }
	}
}

impl TryFrom<u32> for CanExtendedId {
	type Error = InvalidId;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<u8> for CanId {
	fn from(value: u8) -> Self {
		Self::Base(value.into())
	}
}

impl From<u16> for CanId {
	fn from(value: u16) -> Self {
		if value <= MAX_CAN_ID_BASE {
			CanBaseId { id: value }.into()
		} else {
			CanExtendedId::from(value).into()
		}
	}
}

impl TryFrom<u32> for CanId {
	type Error = InvalidId;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<CanBaseId> for CanId {
	fn from(value: CanBaseId) -> Self {
		Self::Base(value)
	}
}

impl From<CanExtendedId> for CanId {
	fn from(value: CanExtendedId) -> Self {
		Self::Extended(value)
	}
}

impl From<CanBaseId> for CanExtendedId {
	fn from(value: CanBaseId) -> Self {
		Self::from(value.as_u16())
	}
}

impl std::str::FromStr for CanBaseId {
	type Err = ParseIdError;

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		let id = parse_number(input)
			.map_err(|e| ParseIdError::invalid_format(e, true))?;
		let id: u16 = id.try_into()
			.map_err(|_| ParseIdError::invalid_value(InvalidId { id: Some(id), extended: false }))?;
		let id = id.try_into()
			.map_err(|_| ParseIdError::invalid_value(InvalidId { id: Some(id.into()), extended: false }))?;
		Ok(id)
	}
}

impl std::str::FromStr for CanExtendedId {
	type Err = ParseIdError;

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		let id = parse_number(input)
			.map_err(|e| ParseIdError::invalid_format(e, true))?;
		let id = id.try_into()
			.map_err(|_| ParseIdError::invalid_value(InvalidId { id: Some(id), extended: true }))?;
		Ok(id)
	}
}

impl std::str::FromStr for CanId {
	type Err = ParseIdError;

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		let id = parse_number(input)
			.map_err(|e| ParseIdError::invalid_format(e, true))?;
		let id = id.try_into()
			.map_err(|_| ParseIdError::invalid_value(InvalidId { id: Some(id), extended: true }))?;
		Ok(id)
	}
}

fn parse_number(input: &str) -> Result<u32, std::num::ParseIntError> {
	if let Some(hexadecimal) = input.strip_prefix("0x") {
		u32::from_str_radix(hexadecimal, 16)
	} else if let Some(octal) = input.strip_prefix("0o") {
		u32::from_str_radix(octal, 8)
	} else if let Some(binary) = input.strip_prefix("0b") {
		u32::from_str_radix(binary, 2)
	} else {
		input.parse()
	}
}

impl std::fmt::Debug for CanId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Base(id) => id.fmt(f),
			Self::Extended(id) => id.fmt(f),
		}
	}
}

impl std::fmt::Debug for CanBaseId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("CanBaseId")
			.field(&format_args!("0x{:03X}", self.id))
			.finish()
	}
}

impl std::fmt::Debug for CanExtendedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("CanExtendedId")
			.field(&format_args!("0x{:08X}", self.id))
			.finish()
	}
}

impl std::fmt::LowerHex for CanBaseId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_u16().fmt(f)
	}
}

impl std::fmt::LowerHex for CanExtendedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_u32().fmt(f)
	}
}

impl std::fmt::LowerHex for CanId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Base(x) => x.fmt(f),
			Self::Extended(x) => x.fmt(f),
		}
	}
}

impl std::fmt::UpperHex for CanBaseId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_u16().fmt(f)
	}
}

impl std::fmt::UpperHex for CanExtendedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_u32().fmt(f)
	}
}

impl std::fmt::UpperHex for CanId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Base(x) => x.fmt(f),
			Self::Extended(x) => x.fmt(f),
		}
	}
}
