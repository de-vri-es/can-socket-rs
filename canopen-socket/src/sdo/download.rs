use can_socket::CanFrame;
use std::time::Duration;

use crate::{CanOpenSocket, ObjectIndex};

use super::{
	ClientCommand,
	SdoAddress,
	SdoError,
	ServerCommand,
	check_server_command,
};

/// Perform an SDO download (write) to the server.
pub async fn sdo_download(
	bus: &mut CanOpenSocket,
	address: SdoAddress,
	node_id: u8,
	object: ObjectIndex,
	data: &[u8],
	timeout: Duration,
) -> Result<(), SdoError> {
	// Can write in a single frame.
	if data.len() <= 4 {
		sdo_download_expedited(
			bus,
			address,
			node_id,
			object,
			data,
			timeout,
		).await
	} else {
		sdo_download_segmented(
			bus,
			address,
			node_id,
			object,
			data,
			timeout,
		).await
	}
}

/// Perform an expedited SDO download (write) to the server.
async fn sdo_download_expedited(
	bus: &mut CanOpenSocket,
	address: SdoAddress,
	node_id: u8,
	object: ObjectIndex,
	data: &[u8],
	timeout: Duration,
) -> Result<(), SdoError> {
	log::debug!("Sending initiate expedited download request");
	log::debug!("├─ SDO: command: 0x{:04X}, response: 0x{:04X}", address.command_address(), address.response_address());
	log::debug!("├─ Node ID: {node_id:?}");
	log::debug!("├─ Object: index = 0x{:04X}, subindex = 0x{:02X}", object.index, object.subindex);
	log::debug!("└─ Timeout: {timeout:?}");
	let command = make_sdo_expedited_download_command(address, node_id, object, data);
	bus.socket.send(&command).await
		.map_err(SdoError::SendFailed)?;

	let response = bus.recv_new_by_can_id(address.response_id(node_id), timeout)
		.await
		.map_err(SdoError::RecvFailed)?
		.ok_or(SdoError::Timeout)?;

	check_server_command(&response, ServerCommand::InitiateDownload)?;
	Ok(())
}

/// Perform an segmented SDO download (write) to the server.
async fn sdo_download_segmented(
	bus: &mut CanOpenSocket,
	address: SdoAddress,
	node_id: u8,
	object: ObjectIndex,
	data: &[u8],
	timeout: Duration,
) -> Result<(), SdoError> {
	let data_len: u32 = data.len().try_into()
		.map_err(|_| super::DataLengthExceedsMaximum { data_len: data.len() })?;

	log::debug!("Sending initiate segmented download request");
	log::debug!("├─ SDO: command: 0x{:04X}, response: 0x{:04X}", address.command_address(), address.response_address());
	log::debug!("├─ Node ID: {node_id:?}");
	log::debug!("├─ Object: index = 0x{:04X}, subindex = 0x{:02X}", object.index, object.subindex);
	log::debug!("├─ Data length: 0x{data_len:04X}");
	log::debug!("└─ Timeout: {timeout:?}");

	// Send command to iniate segmented download to server.
	let command = make_sdo_initiate_segmented_download_command(address, node_id, object, data_len);
	bus.socket.send(&command).await
		.map_err(SdoError::SendFailed)?;

	// Parse response from server.
	let response = bus.recv_new_by_can_id(address.response_id(node_id), timeout)
		.await
		.map_err(SdoError::RecvFailed)?
		.ok_or(SdoError::Timeout)?;
	check_server_command(&response, ServerCommand::InitiateDownload)?;
	log::debug!("Received SDO initiate segmented download response");

	// Download individual chunks to the server.
	let result = async {
		let chunks = data.chunks(7).enumerate();
		let chunk_count = chunks.len();
		for (i, data) in chunks {
			// Send command to download next chunk.
			log::debug!("Sending SDO segment download request to node 0x{node_id:02X}");
			let complete = i + 1 == chunk_count;
			let toggle = i % 2 == 1;
			let command = make_sdo_segment_download_command(address, node_id, toggle, complete, data);
			bus.socket.send(&command).await.map_err(SdoError::SendFailed)?;

			// Parse response.
			let response = bus.recv_new_by_can_id(address.response_id(node_id), timeout)
				.await
				.map_err(SdoError::RecvFailed)?
				.ok_or(SdoError::Timeout)?;
			parse_segment_download_response(&response, toggle)?;
			log::debug!("Received SDO segment download response");
		}
		Ok(())
	}.await;

	match result {
		Err(e) => {
			super::send_abort_transfer_command(
				bus,
				address,
				node_id,
				object,
				crate::sdo::AbortReason::GeneralError,
			).await.ok();
			Err(e)
		},
		Ok(x) => Ok(x),
	}
}

/// Make an SDO initiate expedited download command.
#[allow(clippy::get_first)]
fn make_sdo_expedited_download_command(
	address: SdoAddress,
	node_id: u8,
	object: ObjectIndex,
	data: &[u8],
) -> CanFrame {
	debug_assert!(data.len() <= 4);
	let n = 4 - data.len() as u8;
	let object_index = object.index.to_le_bytes();
	let data: [u8; 8] = [
		u8::from(ClientCommand::InitiateDownload) << 5 | n << 2 | 0x03, // 0x03 means expedited and size-set flags enabled.
		object_index[0],
		object_index[1],
		object.subindex,
		data.get(0).copied().unwrap_or(0),
		data.get(1).copied().unwrap_or(0),
		data.get(2).copied().unwrap_or(0),
		data.get(3).copied().unwrap_or(0),
	];

	CanFrame::new(address.command_id(node_id), &data, None).unwrap()
}

/// Make an SDO initiate segmented download command.
fn make_sdo_initiate_segmented_download_command(
	address: SdoAddress,
	node_id: u8,
	object: ObjectIndex,
	len: u32,
) -> CanFrame {
	let len = len.to_le_bytes();
	let object_index = object.index.to_le_bytes();
	let data: [u8; 8] = [
		u8::from(ClientCommand::InitiateDownload) << 5 | 0x01, // 0x01 means not expedited, size-set enabled.
		object_index[0],
		object_index[1],
		object.subindex,
		len[0],
		len[1],
		len[2],
		len[3],
	];

	CanFrame::new(address.command_id(node_id), &data, None).unwrap()
}

/// Make an SDO download segment command.
#[allow(clippy::get_first)]
fn make_sdo_segment_download_command(
	address: SdoAddress,
	node_id: u8,
	toggle: bool,
	complete: bool,
	data: &[u8],
) -> CanFrame {
	debug_assert!(data.len() <= 7);
	let ccs = u8::from(ClientCommand::SegmentDownload);
	let t = u8::from(toggle);
	let n = 7 - data.len() as u8;
	let c = u8::from(complete);
	let data: [u8; 8] = [
		ccs << 5 | t << 4 | n << 1 | c, // 0x01 means not expedited, size-set enabled.
		data.get(0).copied().unwrap_or(0),
		data.get(1).copied().unwrap_or(0),
		data.get(2).copied().unwrap_or(0),
		data.get(3).copied().unwrap_or(0),
		data.get(4).copied().unwrap_or(0),
		data.get(5).copied().unwrap_or(0),
		data.get(6).copied().unwrap_or(0),
	];
	CanFrame::new(address.command_id(node_id), &data, None).unwrap()
}

/// Parse an SDO download segment response.
fn parse_segment_download_response(frame: &CanFrame, expected_toggle: bool) -> Result<(), SdoError> {
	check_server_command(frame, ServerCommand::SegmentDownload)?;
	let data = frame.data();

	let toggle = data[0] & 0x10 != 0;
	if toggle != expected_toggle {
		return Err(SdoError::InvalidToggleFlag);
	}

	Ok(())
}
