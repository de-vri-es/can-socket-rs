use std::time::Duration;

use can_socket::CanFrame;

use crate::CanOpenSocket;

use super::{
	ClientCommand,
	SdoAddress,
	SdoError,
	ServerCommand,
	check_server_command,
};

pub async fn write_sdo(
	bus: &mut CanOpenSocket,
	address: SdoAddress,
	node_id: u8,
	object_index: u16,
	object_subindex: u8,
	data: &[u8],
	timeout: Duration,
) -> Result<(), SdoError> {
	// Can write in a single frame.
	if data.len() <= 4 {
		return write_sdo_expidited(bus, address, node_id, object_index, object_subindex, data, timeout).await;
	}
	todo!();
}

async fn write_sdo_expidited(
	bus: &mut CanOpenSocket,
	address: SdoAddress,
	node_id: u8,
	object_index: u16,
	object_subindex: u8,
	data: &[u8],
	timeout: Duration,
) -> Result<(), SdoError> {
	let data = make_sdo_data(ClientCommand::InitiateDownload, true, true, object_index, object_subindex, data);
	let frame = CanFrame::new(address.command_id(node_id), &data, None).unwrap();

	bus.socket.send(&frame).await
		.map_err(SdoError::SendFailed)?;

	let response = bus.recv_new_by_can_id(address.response_id(node_id), timeout)
		.await
		.map_err(SdoError::RecvFailed)?
		.ok_or(SdoError::Timeout)?;

	check_server_command(&response, ServerCommand::InitiateDownload)?;
	Ok(())
}

#[allow(clippy::get_first, clippy::identity_op)]
fn make_sdo_data(command: ClientCommand, expedited: bool, size_set: bool, object_index: u16, object_subindex: u8, data: &[u8]) -> [u8; 8] {
	debug_assert!(data.len() <= 4);
	let ccs = command as u8;
	let n = data.len() as u8;
	let expedited: u8 = expedited.into();
	let size_set: u8 = size_set.into();
	[
		ccs << 5 | n << 2 | expedited << 1 | size_set,
		(object_index >> 8 & 0xFF) as u8,
		(object_index >> 0 & 0xFF) as u8,
		object_subindex,
		data.get(0).copied().unwrap_or(0),
		data.get(1).copied().unwrap_or(0),
		data.get(2).copied().unwrap_or(0),
		data.get(3).copied().unwrap_or(0),
	]
}

fn parse_sdo_download_confirmation(frame: &CanFrame) -> Result<(), SdoError> {
	check_server_command(frame, ServerCommand::InitiateDownload)
}
