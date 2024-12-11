use can_socket::{CanData, CanFrame};
use std::{time::Duration, convert::Infallible};

use crate::{CanOpenSocket, ObjectIndex};
use super::{
	SdoAddress,
	ClientCommand,
	SdoError,
	ServerCommand,
	check_server_command,
};

/// Trait for types that can be uploaded from an SDO server.
pub trait UploadObject: Sized {
	/// The size of the object, if known.
	type Buffer: UploadBuffer + Default;

	/// The error type.
	type Error;

	/// Upload the object from the server.
	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error>;
}

/// Trait for types that can be used as a buffer for receiving SDO data.
pub trait UploadBuffer {
	/// Reserve enough space in the buffer.
	///
	/// If the buffer can not be resized and it is too small to hold the needed data,
	/// this function must return a [`super::BufferTooSmall`] error.
	fn reserve(&mut self, needed: usize) -> Result<(), super::BufferTooSmall>;

	/// Add data to the buffer.
	///
	/// The caller guarantees that the total data appended does not exceed the reserved size.
	///
	/// This function may panic if the data does exceed the reserved size.
	fn append(&mut self, data: &[u8]);
}

/// Perform a SDO upload from the server.
pub(crate) async fn sdo_upload<Buffer: UploadBuffer>(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	object: ObjectIndex,
	buffer: &mut Buffer,
	timeout: Duration,
) -> Result<usize, SdoError> {
	log::debug!("Sending initiate upload request");
	log::debug!("├─ Node ID: {node_id:?}");
	log::debug!("├─ SDO: command: 0x{:04X}, response: 0x{:04X}", sdo.command_address(), sdo.response_address());
	log::debug!("├─ Object: index = 0x{:04X}, subindex = 0x{:02X}", object.index, object.subindex);
	log::debug!("└─ Timeout: {timeout:?}");
	let command = make_sdo_initiate_upload_request(node_id, sdo, object);
	bus.socket.send(&command).await
		.map_err(SdoError::SendFailed)?;

	let result: Result<usize, SdoError> = async {
		let response = bus.recv_new_by_can_id(sdo.response_id(node_id), timeout)
			.await
			.map_err(SdoError::RecvFailed)?
			.ok_or(SdoError::Timeout)?;

		let len = match InitiateUploadResponse::parse(&response)? {
			InitiateUploadResponse::Expedited(data) => {
				log::debug!("Received SDO expedited upload response");
				log::debug!("└─ Data: {data:02X?}");
				buffer.reserve(data.len())?;
				buffer.append(&data);
				return Ok(data.len());
			}
			InitiateUploadResponse::Segmented(len) => {
				log::debug!("Received SDO initiate segmented upload response from node 0x{node_id:02X} with data length 0x{len:04X}");
				len as usize
			},
		};

		buffer.reserve(len)?;
		let mut total_len = 0;

		let mut toggle = false;
		loop {
			log::debug!("Sending SDO segment upload request to node 0x{node_id:02X}");
			let command = make_sdo_upload_segment_request(sdo, node_id, toggle);
			bus.socket.send(&command)
				.await
				.map_err(SdoError::SendFailed)?;
			let response = bus.recv_new_by_can_id(sdo.response_id(node_id), timeout)
				.await
				.map_err(SdoError::RecvFailed)?
				.ok_or(SdoError::Timeout)?;
			let (complete, segment_data) = parse_segment_upload_response(&response, toggle)?;
			log::debug!("Received SDO segment upload response");
			log::debug!("├─ Data: {segment_data:02X?}");
			log::debug!("└─ Last segment needed: {complete}");

			if total_len + segment_data.len() >= len as usize {
				return Err(super::WrongDataCount {
					expected: len,
					actual: total_len + segment_data.len(),
				}.into())
			}
			buffer.append(&segment_data);
			total_len += segment_data.len();

			if complete {
				break;
			}

			toggle = !toggle;
		}

		if total_len != len {
			return Err(super::WrongDataCount {
				expected: len,
				actual: total_len,
			}.into());
		}

		Ok(total_len)
	}.await;

	match result {
		Err(e) => {
			super::send_abort_transfer_command(
				bus,
				sdo,
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
	node_id: u8,
	sdo: SdoAddress,
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
	CanFrame::new(sdo.command_id(node_id), data)
}

/// Make an SDO upload segment request.
fn make_sdo_upload_segment_request(address: SdoAddress, node_id: u8, toggle: bool) -> CanFrame {
	let data = [
		(ClientCommand::SegmentUpload as u8) << 5 | u8::from(toggle) << 4,
		0, 0, 0,
		0, 0, 0, 0,
	];
	CanFrame::new(address.command_id(node_id), data)
}

/// An SDO initiate upload response.
enum InitiateUploadResponse {
	/// An expedited response containing the actual data.
	Expedited(CanData),

	/// A segmented response containing the length of the data.
	Segmented(u32),
}

impl InitiateUploadResponse {
	/// Parse an InitiateUploadResponse from a CAN frame.
	fn parse(frame: &CanFrame) -> Result<Self, SdoError> {
		let data = check_server_command(frame, ServerCommand::InitiateUpload)?;

		let n = data[0] >> 2 & 0x03;
		let expedited = data[0] & 0x02 != 0;
		let size_set = data[0] & 0x01 != 0;

		if expedited {
			let len = match size_set {
				true => 4 - n as usize,
				false => 4,
			};
			let data = CanData::try_from(&data[4..][..len]).unwrap();
			Ok(InitiateUploadResponse::Expedited(data))
		} else if !size_set {
			Err(SdoError::NoExpeditedOrSizeFlag)
		} else {
			let len = u32::from_le_bytes(data[4..8].try_into().unwrap());
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
fn parse_segment_upload_response(frame: &CanFrame, expected_toggle: bool) -> Result<(bool, CanData), SdoError> {
	let data = check_server_command(frame, ServerCommand::SegmentUpload)?;

	let toggle = data[0] & 0x10 != 0;
	let n = data[0] >> 1 & 0x07;
	let complete = data[0] & 0x01 != 0;
	let len = 7 - n as usize;

	if toggle != expected_toggle {
		return Err(SdoError::InvalidToggleFlag);
	}

	let data = CanData::try_from(&data[1..][..len]).unwrap();
	Ok((complete, data))
}

impl UploadBuffer for Vec<u8> {
	fn reserve(&mut self, needed: usize) -> Result<(), super::BufferTooSmall> {
		Vec::reserve(self, needed);
		Ok(())
	}

	fn append(&mut self, data: &[u8]) {
		self.extend_from_slice(data)
	}
}

impl UploadBuffer for &mut [u8] {
	fn reserve(&mut self, needed: usize) -> Result<(), super::BufferTooSmall> {
		if needed <= self.len() {
			Ok(())
		} else {
			Err(super::BufferTooSmall {
				available: self.len(),
				needed,
			})
		}
	}

	fn append(&mut self, data: &[u8]) {
		self[..data.len()].copy_from_slice(data);
		*self = &mut std::mem::take(self)[data.len()..];
	}
}

/// Fixed size buffer for uploading small objects from an SDO server.
#[derive(Debug)]
pub struct FixedBuffer<const N: usize> {
	/// The actual buffer.
	data: [u8; N],

	/// The number of bytes written to the buffer so far.
	written: usize,
}

impl<const N: usize> Default for FixedBuffer<N> {
	fn default() -> Self {
		Self {
			data: [0u8; N],
			written: 0,
		}
	}
}

impl<const N: usize> UploadBuffer for FixedBuffer<N> {
	fn reserve(&mut self, needed: usize) -> Result<(), super::BufferTooSmall> {
		if needed <= self.data.len() - self.written {
			Ok(())
		} else {
			Err(super::BufferTooSmall {
				available: self.data.len() - self.written,
				needed,
			})
		}
	}

	fn append(&mut self, data: &[u8]) {
		self.data[self.written..][..data.len()].copy_from_slice(data);
		self.written += data.len()
	}
}

impl UploadObject for Vec<u8> {
	type Buffer = Vec<u8>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(buffer)
	}
}

impl UploadObject for String {
	type Buffer = Vec<u8>;
	type Error = std::string::FromUtf8Error;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		String::from_utf8(buffer)
	}
}

impl<const N: usize> UploadObject for [u8; N] {
	type Buffer = FixedBuffer<N>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(buffer.data)
	}
}

impl UploadObject for u8 {
	type Buffer = FixedBuffer<1>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for i8 {
	type Buffer = FixedBuffer<1>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for u16 {
	type Buffer = FixedBuffer<2>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for i16 {
	type Buffer = FixedBuffer<2>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for u32 {
	type Buffer = FixedBuffer<4>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for i32 {
	type Buffer = FixedBuffer<4>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for u64 {
	type Buffer = FixedBuffer<8>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for i64 {
	type Buffer = FixedBuffer<8>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for u128 {
	type Buffer = FixedBuffer<16>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}

impl UploadObject for i128 {
	type Buffer = FixedBuffer<16>;
	type Error = Infallible;

	fn parse_buffer(buffer: Self::Buffer) -> Result<Self, Self::Error> {
		Ok(Self::from_le_bytes(buffer.data))
	}
}
