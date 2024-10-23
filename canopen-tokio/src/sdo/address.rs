use can_socket::StandardId;

/// The address pair to use for SDO transfers.
#[derive(Debug, Copy, Clone)]
pub struct SdoAddress {
	/// The COB ID (excluding the node ID) to use for sending SDO commands.
	command_address: u16,

	/// The COB ID (excluding the node ID) used by the server to reply to SDO commands.
	response_address: u16,
}

impl SdoAddress {
	/// Make a new SDO address pair with an download (to server) and upload (from server) CANopen COB ID.
	///
	/// Note that download means "download to server" and upload means "upload from server".
	/// Most people outside of [CiA](https://can-cia.org/) would have chosen the opposite meaning.
	pub fn new(command_address: u16, response_address: u16) -> Result<Self, can_socket::error::InvalidId> {
		StandardId::new(command_address)?;
		StandardId::new(response_address)?;
		Ok(Self {
			command_address,
			response_address,
		})
	}

	/// Get the standard SDO address with 0x580 and 0x600 as command and response addresses.
	pub fn standard() -> Self {
		Self {
			command_address: 0x600,
			response_address: 0x580,
		}
	}

	/// Get the full CAN ID for a sending SDO commands to a given node ID.
	pub fn command_id(self, node_id: u8) -> StandardId {
		StandardId::new(self.command_address | u16::from(node_id)).unwrap()
	}

	/// Get the full CAN ID for receiving SDO responses from a given node ID.
	pub fn response_id(self, node_id: u8) -> StandardId {
		StandardId::new(self.response_address | u16::from(node_id)).unwrap()
	}

	/// Get the COB ID (excluding the node ID) to use for sending the SDO commands.
	pub fn command_address(self) -> u16 {
		self.command_address
	}

	/// Get the COB ID (excluding the node ID) used by the server to reply to SDO commands.
	pub fn response_address(self) -> u16 {
		self.response_address
	}
}
