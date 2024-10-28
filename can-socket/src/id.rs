use crate::error::{InvalidId, ParseIdError};

/// The highest valid value for a standard CAN ID.
pub const MAX_STANDARD_ID: u16 = 0x7FF;

/// The highest valid value for an extended CAN ID.
pub const MAX_EXTENDED_ID: u32 = 0x1FFF_FFFF;

/// Construct an [`CanId`] (standard or extended) that is checked at compile time.
///
/// You can use any expression that can be evaluated at compile time and results in a `u32`.
///
/// By default, if the value fits in a standard CAN ID, a standard CAN ID is created.
/// You can also explicitly ask for a standard or extended ID.
///
/// Usage:
/// ```
/// # use can_socket::{CanId, can_id};
/// let id: CanId = can_id!(0x100 | 0x005);
/// assert2::let_assert!(CanId::Standard(id) = id);
/// assert2::assert!(id.as_u16() == 0x105);
/// ```
///
/// Force construction of a `CanId::Standard` (does not compile because the ID only fits as an extended ID):
/// ```compile_fail
/// # use can_socket::{CanId, can_id};
/// let id: CanId = id!(standard: 0x10 << 16 | 0x50);
/// ```
///
/// Force construction of a `CanId::Extended`:
/// ```
/// # use can_socket::{CanId, can_id};
/// let id: CanId = can_id!(extended: 0x100 | 0x005);
/// assert2::let_assert!(CanId::Extended(id) = id);
/// assert2::assert!(id.as_u32() == 0x105);
/// ```
#[macro_export]
macro_rules! can_id {
	($n:expr) => {
		{
			const N: u32 = ($n);
			const { ::core::assert!(N <= $crate::MAX_EXTENDED_ID, "invalid CAN ID") };
			unsafe {
				if N <= $crate::MAX_STANDARD_ID as u32 {
					$crate::CanId::Standard($crate::StandardId::new_unchecked(N as u16))
				} else {
					$crate::CanId::Extended($crate::ExtendedId::new_unchecked(N))
				}
			}
		}
	};
	(standard: $n:expr) => {
		$crate::CanId::Standard($crate::standard_id!($n))
	};
	(extended:  $n:expr) => {
		$crate::CanId::Extended($crate::extended_id!($n))
	};
}

/// Construct a [`StandardId`] that is checked at compile time.
///
/// You can use any expression that can be evaluated at compile time and results in a `u16`.
///
/// Usage:
/// ```
/// # use assert2::assert;
/// # use can_socket::{StandardId, standard_id};
/// let id: StandardId = standard_id!(0x100 | 0x005);
/// assert!(id.as_u16() == 0x105);
/// ```
///
/// Will not accept invalid IDs:
/// ```compile_fail
/// # use can_socket::{StandardId, standard_id};
/// let id: StandardId = standard_id!(0x800);
/// ```
#[macro_export]
macro_rules! standard_id {
	($n:expr) => {
		{
			#[allow(clippy::all)]
			const { ::core::assert!(($n) <= $crate::MAX_STANDARD_ID, "invalid standard CAN ID") };
			unsafe {
				$crate::StandardId::new_unchecked($n)
			}
		}
	};
}

/// Construct an [`ExtendedId`] that is checked at compile time.
///
/// You can use any expression that can be evaluated at compile time and results in a `u32`.
///
/// Usage:
/// ```
/// # use assert2::assert;
/// # use can_socket::{ExtendedId, extended_id};
/// let id: ExtendedId = extended_id!(0x10 << 16 | 0x50);
/// assert!(id.as_u32() == 0x10_0050);
/// ```
///
/// Will not accept invalid IDs:
/// ```compile_fail
/// # use can_socket::{ExtendedId, extended_id};
/// let id: ExtendedId = extended_id!(0x2000_0000);
/// ```
#[macro_export]
macro_rules! extended_id {
	($n:expr) => {
		unsafe {
			#[allow(clippy::all)]
			const { ::core::assert!(($n) <= $crate::MAX_EXTENDED_ID, "invalid extended CAN ID"); };
			$crate::ExtendedId::new_unchecked($n)
		}
	};
}

/// A CAN ID, either standard (11 bit) or extended (29 bits).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub enum CanId {
	/// A standard 11 bit CAN ID.
	Standard(StandardId),

	/// An extended 29 bit CAN ID.
	Extended(ExtendedId),
}

/// A standard 11 bit CAN ID.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct StandardId {
	/// The raw ID.
	id: u16,
}

/// An extended 29 bit CAN ID.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct ExtendedId {
	/// The raw ID.
	id: u32,
}

impl CanId {
	/// Create a new CAN ID from a raw value.
	///
	/// If the value fits in a 11 bit standard CAN ID,
	/// this function will return [`Self::Standard`].
	/// a standard ID will be created.
	///
	/// Otherwise, if the value fits in a 29 bits extended CAN ID,
	/// this function returns [`Self::Extended`].
	///
	/// If the value doesn't fit in either, this function returns an error.
	pub const fn new(id: u32) -> Result<Self, InvalidId> {
		if id <= MAX_STANDARD_ID as u32 {
			let id = id as u16;
			Ok(Self::Standard(StandardId { id }))
		} else {
			match ExtendedId::new(id) {
				Ok(x) => Ok(Self::Extended(x)),
				Err(e) => Err(e)
			}
		}
	}

	/// Create a new standard CAN ID from a raw value.
	///
	/// If the value doesn't fit in a standard 11 bit CAN ID,
	/// this function returns an error.
	pub const fn new_standard(id: u16) -> Result<Self, InvalidId> {
		match StandardId::new(id) {
			Ok(x) => Ok(Self::Standard(x)),
			Err(e) => Err(e),
		}
	}

	/// Create a new extended CAN ID from a raw value.
	///
	/// If the value doesn't fit in an extended 29 bit CAN ID,
	/// this function returns an error.
	pub const fn new_extended(id: u32) -> Result<Self, InvalidId> {
		match ExtendedId::new(id) {
			Ok(x) => Ok(Self::Extended(x)),
			Err(e) => Err(e),
		}
	}

	/// Get the raw value as a `u32`.
	pub const fn as_u32(self) -> u32 {
		self.to_extended().as_u32()
	}

	/// Get `self` as a `StandardId`, or `None` if this is an extended ID.
	///
	/// Note: This function always returns `None` if `self` is an extended ID.
	/// It doesn't matter if the value would have fit in a [`StandardId`].
	///
	/// Use [`Self::to_standard()`] if you want to try to convert extended IDs to standard IDs.
	pub const fn as_standard(self) -> Option<StandardId> {
		match self {
			Self::Standard(id) => Some(id),
			Self::Extended(_) => None,
		}
	}

	/// Get `self` as an `ExtendedId`, or `None` if this is a standard ID.
	///
	/// Use [`Self::to_extended()`] if you want to convert standard IDs to extended IDs.
	pub const fn as_extended(self) -> Option<ExtendedId> {
		match self {
			Self::Standard(_) => None,
			Self::Extended(id) => Some(id),
		}
	}

	/// Try to convert the ID to a standard ID.
	///
	/// Returns an error if the value doesn't fit in a standard ID.
	pub const fn to_standard(self) -> Result<StandardId, InvalidId> {
		match self {
			Self::Standard(id) => Ok(id),
			Self::Extended(id) => {
				if id.as_u32() <= u16::MAX as u32 {
					StandardId::new(id.as_u32() as u16)
				} else {
					Err(InvalidId {
						id: Some(id.as_u32()),
						extended: false,
					})
				}
			}
		}
	}

	/// Convert the ID to an extended ID.
	pub const fn to_extended(self) -> ExtendedId {
		match self {
			Self::Standard(id) => ExtendedId::from_u16(id.as_u16()),
			Self::Extended(id) => id,
		}
	}
}

impl StandardId {
	/// The maximum valid standard ID.
	pub const MAX: Self = Self { id: MAX_STANDARD_ID };

	/// Try to create a new standard ID from a raw value.
	///
	/// Returns an error if the value doesn't fit in 11 bits.
	pub const fn new(id: u16) -> Result<Self, InvalidId> {
		if id <= MAX_STANDARD_ID {
			Ok(Self { id })
		} else {
			Err(InvalidId {
				id: Some(id as u32),
				extended: false,
			})
		}
	}

	/// Create a new standard ID from a `u8`.
	///
	/// Note that [`StandardId`] also implements `From<u8>`.
	/// However, this function is usable in `const` context.
	pub const fn from_u8(id: u8) -> Self {
		Self { id: id as u16 }
	}

	/// Get the raw value as a `u16`.
	///
	/// Note that [`StandardId`] also implements `Into<u16>`.
	/// However, this function is usable in `const` context.
	pub const fn as_u16(self) -> u16 {
		self.id
	}

	/// Create a new standard CAN ID without checking for validity.
	///
	/// # Safety
	/// The given ID must be a valid standard CAN ID (id <= [`MAX_STANDARD_ID`]).
	pub const unsafe fn new_unchecked(id: u16) -> Self {
		debug_assert!(id <= MAX_STANDARD_ID);
		Self {
			id
		}
	}
}

impl ExtendedId {
	/// The maximum valid extended ID.
	pub const MAX: Self = Self { id: MAX_EXTENDED_ID };

	/// Try to create a new extended ID from a raw value.
	///
	/// Returns an error if the value doesn't fit in 29 bits.
	pub const fn new(id: u32) -> Result<Self, InvalidId> {
		if id <= MAX_EXTENDED_ID {
			Ok(Self { id })
		} else {
			Err(InvalidId {
				id: Some(id),
				extended: false,
			})
		}
	}

	/// Create a new extended ID from a `u8`.
	///
	/// Note that [`ExtendedId`] also implements `From<u8>`.
	/// However, this function is usable in `const` context.
	pub const fn from_u8(id: u8) -> Self {
		Self { id: id as u32 }
	}

	/// Create a new extended ID from a `u16`.
	///
	/// Note that [`ExtendedId`] also implements `From<u16>`.
	/// However, this function is usable in `const` context.
	pub const fn from_u16(id: u16) -> Self {
		Self { id: id as u32 }
	}

	/// Get the raw value as a `u32`.
	///
	/// Note that [`ExtendedId`] also implements `Into<u32>`.
	/// However, this function is usable in `const` context.
	pub const fn as_u32(self) -> u32 {
		self.id
	}

	/// Create a new extended CAN ID without checking for validity.
	///
	/// # Safety
	/// The given ID must be a valid extended CAN ID (id <= [`MAX_EXTENDED_ID`]).
	pub const unsafe fn new_unchecked(id: u32) -> Self {
		debug_assert!(id <= MAX_EXTENDED_ID);
		Self {
			id
		}
	}
}

impl PartialEq<StandardId> for CanId {
	fn eq(&self, other: &StandardId) -> bool {
		self.as_standard().is_some_and(|x| x == *other)
	}
}

impl PartialEq<CanId> for StandardId {
	fn eq(&self, other: &CanId) -> bool {
		other == self
	}
}

impl PartialEq<ExtendedId> for CanId {
	fn eq(&self, other: &ExtendedId) -> bool {
		self.as_extended().is_some_and(|x| x == *other)
	}
}

impl PartialEq<CanId> for ExtendedId {
	fn eq(&self, other: &CanId) -> bool {
		other == self
	}
}

impl From<u8> for StandardId {
	fn from(value: u8) -> Self {
		Self { id: value.into() }
	}
}

impl TryFrom<u16> for StandardId {
	type Error = InvalidId;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl TryFrom<u32> for StandardId {
	type Error = InvalidId;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		if value > MAX_STANDARD_ID.into() {
			Err(InvalidId {
				id: Some(value),
				extended: false,
			})
		} else {
			Ok(Self { id: value as u16 })
		}
	}
}

impl TryFrom<ExtendedId> for StandardId {
	type Error = InvalidId;

	fn try_from(value: ExtendedId) -> Result<Self, Self::Error> {
		Self::try_from(value.as_u32())
	}
}

impl TryFrom<CanId> for StandardId {
	type Error = InvalidId;

	fn try_from(value: CanId) -> Result<Self, Self::Error> {
		Self::try_from(value.as_u32())
	}
}

impl From<u8> for ExtendedId {
	fn from(value: u8) -> Self {
		Self { id: value.into() }
	}
}

impl From<u16> for ExtendedId {
	fn from(value: u16) -> Self {
		Self { id: value.into() }
	}
}

impl TryFrom<u32> for ExtendedId {
	type Error = InvalidId;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<StandardId> for ExtendedId {
	fn from(value: StandardId) -> Self {
		value.as_u16().into()
	}
}

impl From<CanId> for ExtendedId {
	fn from(value: CanId) -> Self {
		value.to_extended()
	}
}

impl From<u8> for CanId {
	fn from(value: u8) -> Self {
		Self::Standard(value.into())
	}
}

impl From<u16> for CanId {
	fn from(value: u16) -> Self {
		if value <= MAX_STANDARD_ID {
			StandardId { id: value }.into()
		} else {
			ExtendedId::from(value).into()
		}
	}
}

impl TryFrom<u32> for CanId {
	type Error = InvalidId;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl From<StandardId> for CanId {
	fn from(value: StandardId) -> Self {
		Self::Standard(value)
	}
}

impl From<ExtendedId> for CanId {
	fn from(value: ExtendedId) -> Self {
		Self::Extended(value)
	}
}

impl std::str::FromStr for StandardId {
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

impl std::str::FromStr for ExtendedId {
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
			Self::Standard(id) => id.fmt(f),
			Self::Extended(id) => id.fmt(f),
		}
	}
}

impl std::fmt::Debug for StandardId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("StandardId")
			.field(&format_args!("0x{:03X}", self.id))
			.finish()
	}
}

impl std::fmt::Debug for ExtendedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("ExtendedId")
			.field(&format_args!("0x{:08X}", self.id))
			.finish()
	}
}

impl std::fmt::LowerHex for StandardId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_u16().fmt(f)
	}
}

impl std::fmt::LowerHex for ExtendedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_u32().fmt(f)
	}
}

impl std::fmt::LowerHex for CanId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Standard(x) => x.fmt(f),
			Self::Extended(x) => x.fmt(f),
		}
	}
}

impl std::fmt::UpperHex for StandardId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_u16().fmt(f)
	}
}

impl std::fmt::UpperHex for ExtendedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_u32().fmt(f)
	}
}

impl std::fmt::UpperHex for CanId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Standard(x) => x.fmt(f),
			Self::Extended(x) => x.fmt(f),
		}
	}
}
