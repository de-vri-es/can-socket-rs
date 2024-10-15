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
	pub const fn new(id: u32) -> Result<Self, InvalidId> {
		if id <= MAX_CAN_ID_BASE as u32 {
			let id = id as u16;
			Ok(Self::Base(CanBaseId { id }))
		} else {
			match CanExtendedId::new(id) {
				Ok(x) => Ok(Self::Extended(x)),
				Err(e) => Err(e)
			}
		}
	}

	pub const fn new_base(id: u16) -> Result<Self, InvalidId> {
		match CanBaseId::new(id) {
			Ok(x) => Ok(Self::Base(x)),
			Err(e) => Err(e),
		}
	}

	pub const fn new_extended(id: u32) -> Result<Self, InvalidId> {
		match CanExtendedId::new(id) {
			Ok(x) => Ok(Self::Extended(x)),
			Err(e) => Err(e),
		}
	}

	pub const fn as_u32(self) -> u32 {
		self.to_extended().as_u32()
	}

	pub const fn as_base(self) -> Option<CanBaseId> {
		match self {
			Self::Base(id) => Some(id),
			Self::Extended(_) => None,
		}
	}

	pub const fn as_extended(self) -> Option<CanExtendedId> {
		match self {
			Self::Base(_) => None,
			Self::Extended(id) => Some(id),
		}
	}

	pub const fn to_base(self) -> Result<CanBaseId, InvalidId> {
		match self {
			Self::Base(id) => Ok(id),
			Self::Extended(id) => {
				if id.as_u32() <= u16::MAX as u32 {
					CanBaseId::new(id.as_u32() as u16)
				} else {
					Err(InvalidId {
						id: Some(id.as_u32()),
						extended: false,
					})
				}
			}
		}
	}

	pub const fn to_extended(self) -> CanExtendedId {
		match self {
			Self::Base(id) => CanExtendedId::from_u16(id.as_u16()),
			Self::Extended(id) => id,
		}
	}
}

impl CanBaseId {
	pub const fn new(id: u16) -> Result<Self, InvalidId> {
		if id <= MAX_CAN_ID_BASE {
			Ok(Self { id })
		} else {
			Err(InvalidId {
				id: Some(id as u32),
				extended: false,
			})
		}
	}

	pub const fn from_u8(id: u8) -> Self {
		Self { id: id as u16 }
	}

	pub const fn as_u16(self) -> u16 {
		self.id
	}
}

impl CanExtendedId {
	pub const fn new(id: u32) -> Result<Self, InvalidId> {
		if id <= MAX_CAN_ID_EXTENDED {
			Ok(Self { id })
		} else {
			Err(InvalidId {
				id: Some(id),
				extended: false,
			})
		}
	}

	pub const fn from_u8(id: u8) -> Self {
		Self { id: id as u32 }
	}

	pub const fn from_u16(id: u16) -> Self {
		Self { id: id as u32 }
	}

	pub const fn as_u32(self) -> u32 {
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

impl TryFrom<u32> for CanBaseId {
	type Error = InvalidId;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		if value > MAX_CAN_ID_BASE.into() {
			Err(InvalidId {
				id: Some(value),
				extended: false,
			})
		} else {
			Ok(Self { id: value as u16 })
		}
	}
}

impl TryFrom<CanExtendedId> for CanBaseId {
	type Error = InvalidId;

	fn try_from(value: CanExtendedId) -> Result<Self, Self::Error> {
		Self::try_from(value.as_u32())
	}
}

impl TryFrom<CanId> for CanBaseId {
	type Error = InvalidId;

	fn try_from(value: CanId) -> Result<Self, Self::Error> {
		Self::try_from(value.as_u32())
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

impl From<CanBaseId> for CanExtendedId {
	fn from(value: CanBaseId) -> Self {
		value.as_u16().into()
	}
}

impl From<CanId> for CanExtendedId {
	fn from(value: CanId) -> Self {
		value.to_extended()
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
