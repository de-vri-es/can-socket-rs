use can_socket::CanFrame;
use std::time::Duration;

use crate::{CanOpenSocket, ObjectIndex};
use super::{
	SdoAddress,
	ClientCommand,
	SdoError,
	ServerCommand,
	WrongDataCount,
	check_server_command,
};


/// Perform a SDO upload from the server.
pub async fn sdo_upload(
	bus: &mut CanOpenSocket,
	address: SdoAddress,
	node_id: u8,
	object: ObjectIndex,
	timeout: Duration,
) -> Result<Vec<u8>, SdoError> {
	log::debug!("Sending initiate upload request");
	log::debug!("├─ SDO: command: 0x{:04X}, response: 0x{:04X}", address.command_address(), address.response_address());
	log::debug!("├─ Node ID: {node_id:?}");
	log::debug!("├─ Object: index = 0x{:04X}, subindex = 0x{:02X}", object.index, object.subindex);
	log::debug!("└─ Timeout: {timeout:?}");
	let command = make_sdo_initiate_upload_request(address, node_id, object);
	bus.socket.send(&command).await
		.map_err(SdoError::SendFailed)?;

	let result = async {
		let response = bus.recv_new_by_can_id(address.response_id(node_id), timeout)
			.await
			.map_err(SdoError::RecvFailed)?
			.ok_or(SdoError::Timeout)?;

		let len = match InitiateUploadResponse::parse(&response)? {
			InitiateUploadResponse::Expedited(data) => {
				log::debug!("Received SDO expedited upload response from node 0x{node_id:02X}: {data:02X?}");
				return Ok(data.into());
			}
			InitiateUploadResponse::Segmented(len) => {
				log::debug!("Received SDO initiate segmented upload response from node 0x{node_id:02X} with data length 0x{len:04X}");
				len as usize
			},
		};

		let mut data = Vec::with_capacity(len);
		let mut toggle = false;
		loop {
			log::debug!("Sending SDO segment upload request to node 0x{node_id:02X}");
			let command = make_sdo_upload_segment_request(address, node_id, toggle);
			bus.socket.send(&command)
				.await
				.map_err(SdoError::SendFailed)?;
			let response = bus.recv_new_by_can_id(address.response_id(node_id), timeout)
				.await
				.map_err(SdoError::RecvFailed)?
				.ok_or(SdoError::Timeout)?;
			let (complete, segment_data) = parse_segment_upload_response(&response, toggle)?;
			log::debug!("Received SDO segment upload response with data: {segment_data:02X?}");
			log::debug!("└─ Last segment needed: {complete}");
			data.extend_from_slice(segment_data);

			if complete {
				break;
			}

			if data.len() >= len as usize {
				return Err(SdoError::TooManySegments)
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

/// Make an SDO initiate upload request.
fn make_sdo_initiate_upload_request(
	address: SdoAddress,
	node_id: u8,
	object: ObjectIndex,
) -> CanFrame {
	let object_index = object.index.to_le_bytes();
	let data = [
		(ClientCommand::InitiateUpload as u8) << 5,
		object_index[0],
		object_index[1],
		object.subindex,
		0, 0, 0, 0,
	];
	CanFrame::new(address.command_id(node_id), &data, None).unwrap()
}

/// Make an SDO upload segment request.
fn make_sdo_upload_segment_request(address: SdoAddress, node_id: u8, toggle: bool) -> CanFrame {
	let data = [
		(ClientCommand::SegmentUpload as u8) << 5 | u8::from(toggle) << 4,
		0, 0, 0,
		0, 0, 0, 0,
	];
	CanFrame::new(address.command_id(node_id), &data, None).unwrap()
}

/// An SDO initiate upload response.
enum InitiateUploadResponse<'a> {
	/// An expedited response containing the actual data.
	Expedited(&'a [u8]),

	/// A segmented response containing the length of the data.
	Segmented(u32),
}

impl<'a> InitiateUploadResponse<'a> {
	/// Parse an InitiateUploadResponse from a CAN frame.
	fn parse(frame: &'a CanFrame) -> Result<Self, SdoError> {
		check_server_command(frame, ServerCommand::InitiateUpload)?;
		let data = frame.data();

		let n = data[0] >> 2 & 0x03;
		let expedited = data[0] & 0x02 != 0;
		let size_set = data[0] & 0x01 != 0;

		if expedited {
			let len = match size_set {
				true => 4 - n as usize,
				false => 4,
			};
			Ok(InitiateUploadResponse::Expedited(&data[4..][..len]))
		} else if !size_set {
			Err(SdoError::NoExpeditedOrSizeFlag)
		} else {
			let len = u32::from_le_bytes(frame.data()[4..8].try_into().unwrap());
			Ok(InitiateUploadResponse::Segmented(len))
		}
	}
}

/// Parse an SDO segment upload response.
///
/// If successfull, returns a tuple with a boolean and a byte slice.
///
/// The boolean indicates if the transfer is completed by this frame.
/// The byte slice holds the data of the frame.
fn parse_segment_upload_response(frame: &CanFrame, expected_toggle: bool) -> Result<(bool, &[u8]), SdoError> {
	check_server_command(frame, ServerCommand::SegmentUpload)?;
	let data = frame.data();

	let toggle = data[0] & 0x10 != 0;
	let n = data[0] >> 1 & 0x07;
	let complete = data[0] & 0x01 != 0;
	let len = 7 - n as usize;

	if toggle != expected_toggle {
		return Err(SdoError::InvalidToggleFlag);
	}

	Ok((complete, &data[1..][..len]))
}
