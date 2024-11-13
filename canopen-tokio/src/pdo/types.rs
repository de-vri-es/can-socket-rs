use can_socket::CanId;

use crate::{NodeId, ObjectIndex};

use super::InvalidSyncInterval;

/// Configuration of an RPDO,
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RpdoConfiguration {
    /// The communication parameters.
    pub communication: RpdoCommunicationParameters,

    /// The PDO mapping.
    pub mapping: Vec<PdoMapping>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RpdoKind {
    First(NodeId),
    Second(NodeId),
    Third(NodeId),
    Fourth(NodeId),
    // Extended(NodeId, u16)
}

impl RpdoKind {
    pub fn new(node: NodeId, ord: u16) -> Option<Self> {
        let kind = match ord {
            0 => RpdoKind::First(node),
            1 => RpdoKind::Second(node),
            2 => RpdoKind::Third(node),
            3 => RpdoKind::Fourth(node),
            _ => {
                return None;
            }
        };

        Some(kind)
    }

    pub fn ord(&self) -> u16 {
        match self {
            RpdoKind::First(_) => 0,
            RpdoKind::Second(_) => 1,
            RpdoKind::Third(_) => 2,
            RpdoKind::Fourth(_) => 3,
        }
    }
}

impl From<RpdoKind> for CanId {
    fn from(value: RpdoKind) -> Self {
        let id = match value {
            RpdoKind::First(node_id) => 0x200 + node_id as u16,
            RpdoKind::Second(node_id) => 0x300 + node_id as u16,
            RpdoKind::Third(node_id) => 0x400 + node_id as u16,
            RpdoKind::Fourth(node_id) => 0x500 + node_id as u16,
        };

        CanId::new_standard(id).unwrap()
    }
}

/// The communication parameters of an RPDO.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RpdoCommunicationParameters {
    /// If true, the PDO is enabled.
    pub enabled: bool,

    /// The transmission type for the PDO.
    pub mode: RpdoTransmissionType,

    pub cob_id: CanId,

    /// The inhibit time in multiples of 100 microseconds.
    ///
    /// The meaning of the inhibit time is device specific for an RPDO.
    pub inhibit_time_100us: u16,

    /// The deadline timer in milliseconds for the RPDO.
    ///
    /// When the deadline timer expires without, the application will be notifified of the event.
    /// The deadline timer resets every time the PDO is received.
    pub deadline_timer_ms: u16,
}

/// Configuration of an TPDO,
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TpdoConfiguration {
    /// The communication parameters.
    pub communication: TpdoCommunicationParameters,

    /// The PDO mapping.
    pub mapping: Vec<PdoMapping>,
}
/// List of standard  
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TpdoKind {
    First(NodeId),
    Second(NodeId),
    Third(NodeId),
    Fourth(NodeId),
}

impl TpdoKind {
    pub fn new(node: NodeId, ord: u16) -> Option<Self> {
        let kind = match ord {
            0 => TpdoKind::First(node),
            1 => TpdoKind::Second(node),
            2 => TpdoKind::Third(node),
            3 => TpdoKind::Fourth(node),
            _ => {
                return None;
            }
        };

        Some(kind)
    }

    pub fn ord(&self) -> u16 {
        match self {
            TpdoKind::First(_) => 0,
            TpdoKind::Second(_) => 1,
            TpdoKind::Third(_) => 2,
            TpdoKind::Fourth(_) => 3,
        }
    }
}

impl From<TpdoKind> for CanId {
    fn from(value: TpdoKind) -> Self {
        let id = match value {
            TpdoKind::First(node_id) => 0x180 + node_id as u16,
            TpdoKind::Second(node_id) => 0x280 + node_id as u16,
            TpdoKind::Third(node_id) => 0x380 + node_id as u16,
            TpdoKind::Fourth(node_id) => 0x480 + node_id as u16,
        };

        CanId::new_standard(id).unwrap()
    }
}

/// The communication parameters of a TPDO.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TpdoCommunicationParameters {
    /// If true, the PDO is enabled.
    pub enabled: bool,

    /// If true, a remote-transmit requests for the PDO are allowed.
    ///
    /// When enabled, the device will respons with a PDO message in response to the RTR.
    pub rtr_allowed: bool,

    pub cob_id: CanId,

    /// The transmission type of the PDO.
    pub mode: TpdoTransmissionType,

    /// The inhibit time in multiples of 100 microseconds.
    ///
    /// The inihibit time is the minimum interval between two event driven PDO transmissions.
    ///
    /// Ignored for other transmission types.
    pub inhibit_time_100us: u16,

    /// The maximum interval in milliseconds between transmissions for event driven PDOs.
    ///
    /// Ignored for other transmission types.
    pub event_timer_ms: u16,

    /// The SYNC counter value that has to be observer before the first tranmission of this PDO.
    pub start_sync: u8,
}

/// A PDO mapping.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PdoMapping {
    /// The object to add to the PDO.
    pub object: ObjectIndex,

    /// The amount of bits of the object to add in the PDO.
    pub bit_length: u8,
}

impl PdoMapping {
    /// Parse a PDO mapping from a `u32` value as stored in the object dictionary.
    pub fn from_u32(raw: u32) -> Self {
        let index = (raw >> 16 & 0xFFFF) as u16;
        let subindex = (raw >> 8 & 0xFF) as u8;
        let bit_length = (raw & 0xFF) as u8;
        Self {
            object: ObjectIndex::new(index, subindex),
            bit_length,
        }
    }

    /// Get the `u32` value of the PDO mapping as stored in the object dictionary.
    pub fn to_u32(self) -> u32 {
        u32::from(self.object.index) << 16
            | u32::from(self.object.subindex) << 8
            | u32::from(self.bit_length)
    }
}

/// The transmission type of an RPDO.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct RpdoTransmissionType {
    raw: u8,
}

/// The transmission type of an TPDO.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct TpdoTransmissionType {
    raw: u8,
}

impl RpdoTransmissionType {
    /// Create an RPDO transmission type from a raw `u8` value.
    ///
    /// This does not check for reserved values but wraps the value as-is.
    pub const fn from_u8(raw: u8) -> Self {
        Self { raw }
    }

    /// Get the raw `u8` value of the transmission type.
    pub const fn to_u8(self) -> u8 {
        self.raw
    }

    /// Synchronous transmission.
    ///
    /// At every SYNC command, the last read PDO value will be written to the object dictionary.
    pub const fn sync() -> Self {
        Self::from_u8(0)
    }

    /// Check if the transmission mode is synchronous acyclic.
    pub const fn is_sync(self) -> bool {
        self.raw <= 0xF0
    }

    /// Check if the transmission type is one of the reserved values.
    pub const fn is_reserved(&self) -> bool {
        self.raw >= 0xF1 && self.raw <= 0xFB
    }

    /// Event driven transmission.
    ///
    /// The PDO value will directly be written to the object dictionary when the PDO is received.
    ///
    /// To use the manuacturer specific event driven mode, pass `true` for the `manuacturer_specific` parameter.
    pub const fn event_driven(manuacturer_specific: bool) -> Self {
        if manuacturer_specific {
            Self::from_u8(0xFE)
        } else {
            Self::from_u8(0xFF)
        }
    }

    /// Check if the transmission mode is event driven.
    ///
    /// Returns `Some(true)` for manuacturer specific event driven mode.
    /// Returns `Some(false)` for device profile based event driven mode.
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

impl TpdoTransmissionType {
    /// Create an TPDO transmission type from a raw `u8` value.
    ///
    /// This does not check for reserved values but wraps the value as-is.
    pub const fn from_u8(raw: u8) -> Self {
        Self { raw }
    }

    /// Get the raw `u8` value of the transmission type.
    pub const fn to_u8(self) -> u8 {
        self.raw
    }

    /// Synchronous acyclic transmission.
    ///
    /// The PDO will be sent in response to a SYNC command,
    /// but only if an event occured.
    pub const fn sync_acyclic() -> Self {
        Self::from_u8(0)
    }

    /// Check if the transmission mode is synchronous acyclic.
    pub const fn is_sync_acyclic(self) -> bool {
        self.raw == 0
    }

    /// Synchronous transmission.
    ///
    /// The PDO is sent in response to a SYNC command.
    ///
    /// The PDO will be sent every time `interval` SYNC commands have been received.
    /// If the interval is 1, the PDO will be sent in response to every SYNC command.
    pub const fn sync(interval: u8) -> Result<Self, InvalidSyncInterval> {
        if interval >= 1 && interval <= 0xF0 {
            Ok(Self::from_u8(interval))
        } else {
            Err(InvalidSyncInterval { value: interval })
        }
    }

    /// Check if the transmission type is synchronous cyclical.
    ///
    /// Returns the sync interval as `Some(u8)` if the transmission mode is synchronous.
    pub const fn is_sync(&self) -> Option<u8> {
        if self.raw >= 1 && self.raw <= 0xF0 {
            Some(self.raw)
        } else {
            None
        }
    }

    /// Check if the transmission type is one of the reserved values.
    pub const fn is_reserved(&self) -> bool {
        self.raw >= 0xF1 && self.raw <= 0xFB
    }

    /// RTR only.
    ///
    /// The PDO will only be sent in response to a remote-transmission request.
    pub const fn rtr_only(sync: bool) -> Self {
        if sync {
            Self::from_u8(0xFC)
        } else {
            Self::from_u8(0xFD)
        }
    }

    /// Check if the transmission type is RTR only.
    pub const fn is_rtr_only(&self) -> Option<bool> {
        if self.raw == 0xFC {
            Some(true)
        } else if self.raw == 0xFD {
            Some(false)
        } else {
            None
        }
    }

    /// Event driven transmission.
    ///
    /// The PDO will be sent when an event occurs and ignores SYNC commands.
    ///
    /// To use the manuacturer specific event driven mode, pass `true` for the `manuacturer_specific` parameter.
    pub const fn event_driven(manuacturer_specific: bool) -> Self {
        if manuacturer_specific {
            Self::from_u8(0xFE)
        } else {
            Self::from_u8(0xFF)
        }
    }

    /// Check if the transmission mode is event driven.
    ///
    /// Returns `Some(true)` for manuacturer specific event driven mode.
    /// Returns `Some(false)` for device profile based event driven mode.
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

impl std::fmt::Debug for RpdoTransmissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw = self.raw;
        if self.is_sync() {
            write!(f, "Sync(0x{raw:02X})")
        } else if self.is_reserved() {
            write!(f, "Reserved(0x{raw:02X})")
        } else if let Some(manuacturer_specific) = self.is_event_driven() {
            write!(
                f,
                "EventDriven(manuacturer_specific: {manuacturer_specific}, 0x{raw:02X})"
            )
        } else {
            write!(f, "Unknown(0x{raw:02X})")
        }
    }
}

impl std::fmt::Debug for TpdoTransmissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw = self.raw;
        if self.is_sync_acyclic() {
            write!(f, "SyncAcyclic(0x{raw:02X})")
        } else if self.is_sync().is_some() {
            write!(f, "Sync(0x{raw:02X})")
        } else if self.is_reserved() {
            write!(f, "Reserved(0x{raw:02X})")
        } else if let Some(sync) = self.is_rtr_only() {
            write!(f, "RtrOnly(sync: {sync}, 0x{raw:02X})")
        } else if let Some(manuacturer_specific) = self.is_event_driven() {
            write!(
                f,
                "EventDriven(manuacturer_specific: {manuacturer_specific}, 0x{raw:02X})"
            )
        } else {
            write!(f, "Unknown(0x{raw:02X})")
        }
    }
}
