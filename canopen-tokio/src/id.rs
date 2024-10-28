//! CANopen extensions for CAN IDs.

use can_socket::StandardId;

/// CANopen extension for [`CanBaseId`].
///
/// CANopen does not use extended CAN ID's,
/// so the extension trait is only implemented for [`CanBaseId`]
pub trait CanBaseIdExt {
	/// Get the function code from the CAN ID (`can_id & 0x780).
	///
	/// The function code is contained in the 4 bits after the first 7 least significant bits.
	/// The function code is returned unshifted, but with all other bits set to 0.
	fn function_code(&self) -> u16;

	/// Get the node ID part of the CAN ID.
	///
	/// The node ID is the 7 least significant bits of the CAN ID.
	fn node_id(&self) -> u8;
}

impl CanBaseIdExt for StandardId {
	fn function_code(&self) -> u16 {
		self.as_u16() & (0x0F << 7)
	}

	fn node_id(&self) -> u8 {
		(self.as_u16() & 0x7F) as u8
	}
}
