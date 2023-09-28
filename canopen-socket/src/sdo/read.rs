use std::time::Duration;

use can_socket::CanFrame;

use crate::CanOpenSocket;
use super::{
	SdoAddress,
	ClientCommand,
	MalformedResponse,
	SdoError,
	ServerCommand,
	WrongDataCount,
	check_server_command,
};


pub async fn read_sdo(
	bus: &mut CanOpenSocket,
	address: SdoAddress,
	node_id: u8,
	object_index: u16,
	object_subindex: u8,
	timeout: Duration,
) -> Result<Vec<u8>, SdoError> {
	log::debug!("Reading SDO {address:?} from node 0x{node_id:02X} with object index 0x{object_index:04X} and subindex 0x{object_subindex:02X} (timeout {timeout:?})");
	let data = make_sdo_initiate_upload_request(object_index, object_subindex);
	let command = CanFrame::new(address.command_id(node_id), &data, None).unwrap();

	bus.socket.send(&command).await
		.map_err(SdoError::SendFailed)?;

	let response = bus.recv_new_by_can_id(address.response_id(node_id), timeout)
		.await
		.map_err(SdoError::RecvFailed)?
		.ok_or(SdoError::Timeout)?;

	let len = match InitiateUploadResponse::parse(&response)? {
		InitiateUploadResponse::Expedited(data) => {
			log::debug!("Received expidited SDO response from node 0x{node_id:02X}: {data:02X?}");
			return Ok(data.into());
		}
		InitiateUploadResponse::Segmented(len) => {
			log::debug!("Received initial SDO response from node 0x{node_id:02X} with data length 0x{len:04X}");
			len as usize
		},
	};

	let mut data = Vec::with_capacity(len);
	let mut toggle = false;
	loop {
		log::debug!("Sending SDO segment request to node 0x{node_id:02X}");
		let command = CanFrame::new(address.command_id(node_id), &make_sdo_upload_segment_request(toggle), None).unwrap();
		bus.socket.send(&command)
			.await
			.map_err(SdoError::SendFailed)?;
		let response = bus.recv_new_by_can_id(address.response_id(node_id), timeout)
			.await
			.map_err(SdoError::RecvFailed)?
			.ok_or(SdoError::Timeout)?;
		let (more, segment_data) = parse_segment_upload_response(&response, toggle)?;
		log::debug!("Received SDO segment response with data: {segment_data:02X?}");
		log::debug!("    More segments needed: {more}");
		data.extend_from_slice(segment_data);

		if !more {
			break;
		}

		if data.len() >= len as usize {
			return Err(MalformedResponse::TooManySegments.into())
		}

		toggle = !toggle;
	}

	if data.len() != len {
		return Err(WrongDataCount {
			expected: len,
			actual: data.len(),
		}.into())
	}

	Ok(data)
}

fn make_sdo_initiate_upload_request(object_index: u16, object_subindex: u8) -> [u8; 8] {
	let object_index = object_index.to_le_bytes();
	[
		(ClientCommand::InitiateUpload as u8) << 5,
		object_index[0],
		object_index[1],
		object_subindex,
		0, 0, 0, 0,
	]
}

fn make_sdo_upload_segment_request(toggle: bool) -> [u8; 8] {
	[
		(ClientCommand::SegmentUpload as u8) << 5 | u8::from(toggle) << 4,
		0, 0, 0,
		0, 0, 0, 0,
	]
}

enum InitiateUploadResponse<'a> {
	Expedited(&'a [u8]),
	Segmented(u32),
}

impl<'a> InitiateUploadResponse<'a> {
	fn parse(frame: &'a CanFrame) -> Result<Self, SdoError> {
		check_server_command(frame, ServerCommand::InitiateUpload)?;
		let data = frame.data();

		let n = data[0] >> 2 & 0x03;
		let expidited = data[0] & 0x02 != 0;
		let size_set = data[0] & 0x01 != 0;

		if expidited {
			let len = match size_set {
				true => 4 - n as usize,
				false => 4,
			};
			Ok(InitiateUploadResponse::Expedited(&data[4..][..len]))
		} else if !size_set {
			Err(MalformedResponse::InvalidFlags.into())
		} else {
			let len = u32::from_le_bytes(frame.data()[4..8].try_into().unwrap());
			Ok(InitiateUploadResponse::Segmented(len))
		}
	}
}

fn parse_segment_upload_response(frame: &CanFrame, expected_toggle: bool) -> Result<(bool, &[u8]), SdoError> {
	check_server_command(frame, ServerCommand::SegmentUpload)?;
	let data = frame.data();

	let toggle = data[0] & 0x10 != 0;
	let n = data[0] >> 1 & 0x07;
	let done = data[0] & 0x01 != 0;
	let len = 7 - n as usize;

	if toggle != expected_toggle {
		return Err(SdoError::MalformedResponse(MalformedResponse::InvalidToggleFlag));
	}

	Ok((!done, &data[1..][..len]))
}
