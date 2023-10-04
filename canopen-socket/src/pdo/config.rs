use can_socket::CanId;

use crate::sdo::{SdoError, SdoAddress};
use crate::{ObjectIndex, CanOpenSocket};

use super::InvalidNthSyncCounter;

pub(crate) fn read_rpdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo_address: SdoAddress,
	pdo: u16,
) -> Result<RpdoMapping, SdoError> {
	todo!()
}

pub(crate) fn read_tpdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo_address: SdoAddress,
	pdo: u16,
) -> Result<TpdoMapping, SdoError> {
	todo!()
}

pub(crate) fn set_rpdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo_address: SdoAddress,
	pdo: u16,
	mapping: RpdoMapping,
) -> Result<(), SdoError> {
	todo!()
}

pub(crate) fn set_tpdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo_address: SdoAddress,
	pdo: u16,
	mapping: TpdoMapping,
) -> Result<(), SdoError> {
	todo!()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RpdoMapping {
	pub enabled: bool,
	pub cob_id: CanId,
	pub mode: RpdoCommunicationMode,
	pub inhibit_time_100us: u32,
	pub deadline_timer_ms: u32,
	pub content: Vec<PdoField>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TpdoMapping {
	pub enabled: bool,
	pub cob_id: CanId,
	pub mode: TpdoCommunicationMode,
	pub inhibit_time_100us: u32,
	pub event_timer_ms: u32,
	pub start_sync: u8,
	pub content: Vec<PdoField>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PdoField {
	pub object: ObjectIndex,
	pub bit_length: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct RpdoCommunicationMode {
	raw: u8,
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct TpdoCommunicationMode {
	raw: u8,
}

impl RpdoCommunicationMode {
	pub const fn from_u8(raw: u8) -> Self {
		Self { raw }
	}

	pub const fn to_u8(self) -> u8 {
		self.raw
	}
}

impl TpdoCommunicationMode {
	pub const fn from_u8(raw: u8) -> Self {
		Self { raw }
	}

	pub const fn to_u8(self) -> u8 {
		self.raw
	}

	pub const fn sync_acyclic() -> Self {
		Self::from_u8(0)
	}

	pub const fn is_sync_acyclic(self) -> bool {
		self.raw == 0
	}

	pub const fn every_sync() -> Self {
		Self::from_u8(1)
	}

	pub const fn is_every_sync(self) -> bool {
		self.raw == 1
	}

	pub const fn nth_sync(counter: u8) -> Result<Self, InvalidNthSyncCounter> {
		if counter >= 2 && counter <= 0xF0 {
			Ok(Self::from_u8(counter))
		} else {
			Err(InvalidNthSyncCounter { value: counter })
		}
	}

	pub const fn is_nth_sync(&self) -> Option<u8> {
		if self.raw >= 2 && self.raw <= 0xF0 {
			Some(self.raw)
		} else {
			None
		}
	}

	pub const fn is_reserved(&self) -> bool {
		self.raw >= 0xF1 && self.raw <= 0xFB
	}

	pub const fn rtr_only(sync: bool) -> Self {
		if sync {
			Self::from_u8(0xFC)
		} else {
			Self::from_u8(0xFD)
		}
	}

	pub const fn is_rtr_only(&self) -> Option<bool> {
		if self.raw == 0xFC {
			Some(true)
		} else if self.raw == 0xFD {
			Some(false)
		} else {
			None
		}
	}

	pub const fn event_driven(manuacturer_specific: bool) -> Self {
		if manuacturer_specific {
			Self::from_u8(0xFE)
		} else {
			Self::from_u8(0xFF)
		}
	}

	pub const fn is_event_driven(&self) -> Option<bool> {
		if self.raw == 0xFE {
			Some(true)
		} else if self.raw == 0xFF {
			Some(false)
		} else {
			None
		}
	}
}

impl std::fmt::Debug for TpdoCommunicationMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let raw = self.raw;
		if self.is_sync_acyclic() {
			write!(f, "SyncAcyclic(0x{raw:02X})")
		} else if self.is_every_sync() {
			write!(f, "EverySync(0x{raw:02X})")
		} else if self.is_nth_sync().is_some() {
			write!(f, "NthSync(0x{raw:02X})")
		} else if let Some(sync) = self.is_rtr_only() {
			write!(f, "RtrOnly(sync: {sync}, 0x{raw:02X})")
		} else if let Some(manuacturer_specific) = self.is_event_driven() {
			write!(f, "EventDriven(manuacturer_specific: {manuacturer_specific}, 0x{raw:02X})")
		} else {
			write!(f, "Unknown(0x{raw:02X})")
		}
	}
}
