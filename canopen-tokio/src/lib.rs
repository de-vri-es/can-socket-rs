//! CANopen implementation for [`tokio`].

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use can_socket::tokio::CanSocket;
use can_socket::{CanFrame, CanBaseId};
use std::num::NonZeroU8;
use std::time::{Duration, Instant};

mod id;
mod sync;
pub use id::CanBaseIdExt;

pub mod nmt;
pub mod pdo;
pub mod sdo;

/// A CANopen socket.
///
/// Wrapper around a [`CanSocket`] that implements the `CANopen` protocol.
#[allow(missing_debug_implementations)]
pub struct CanOpenSocket {
	socket: CanSocket,
	// TODO: Save messages for later delivery?
	// read_queue: Vec<CanFrame>,
}

/// An index in the object dictionary.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ObjectIndex {
	/// The main index of the object.
	pub index: u16,

	/// The subindex of the object.
	pub subindex: u8,
}

impl CanOpenSocket {
	/// Create a new CANopen socket from a [`CanSocket`].
	pub fn new(can_socket: CanSocket) -> Self {
		Self {
			socket: can_socket,
		}
	}

	/// Receive a raw CAN frame with a deadline.
	///
	/// Returns [`None`] if the deadline expires before a frame arrives.
	/// Returns `Some(Err(...))` if the underlying CAN socket gives an error.
	pub async fn recv_frame_deadline(
		&mut self,
		deadline: Instant,
	) -> Option<std::io::Result<can_socket::CanFrame>> {
		if Instant::now() >= deadline {
			return None;
		}
		tokio::time::timeout_at(deadline.into(), self.socket.recv()).await.ok()
	}

	/// Send a raw CAN frame.
	pub async fn send_frame(
		&mut self,
		frame: &CanFrame,
	) -> std::io::Result<()> {
		self.socket.send(frame).await
	}

	/// Send an NMT command and wait for the device to go into the specified state.
	pub async fn send_nmt_command(
		&mut self,
		node_id: u8,
		command: nmt::NmtCommand,
		timeout: Duration,
	) -> Result<(), nmt::NmtError> {
		nmt::send_nmt_command(self, node_id, command, timeout).await
	}

	/// Read an object dictionary value by performing an upload from a SDO server.
	///
	/// Note that upload means "upload to server".
	/// Most people outside of [CiA](https://can-cia.org/) would call this a download.
	pub async fn sdo_upload_raw(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		object: ObjectIndex,
		buffer: &mut [u8],
		timeout: Duration,
	) -> Result<usize, sdo::SdoError> {
		let mut buffer = buffer;
		sdo::sdo_upload(self, node_id, sdo, object, &mut buffer, timeout).await
	}

	/// Read an object dictionary value by performing an upload from a SDO server.
	///
	/// Note that upload means "upload to server".
	/// Most people outside of [CiA](https://can-cia.org/) would call this a download.
	pub async fn sdo_upload<T: sdo::UploadObject>(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		object: ObjectIndex,
		timeout: Duration,
	) -> Result<T, sdo::UploadError<T::Error>> {
		let mut buffer = <T as sdo::UploadObject>::Buffer::default();
		sdo::sdo_upload(self, node_id, sdo, object, &mut buffer, timeout).await
			.map_err(sdo::UploadError::UploadFailed)?;
		T::parse_buffer(buffer)
			.map_err(sdo::UploadError::ParseFailed)
	}

	/// Write an object dictionary value by performing a download to a SDO server.
	///
	/// Note that download means "download to server".
	/// Most people outside of [CiA](https://can-cia.org/) would call this an upload.
	pub async fn sdo_download<T: sdo::DownloadObject>(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		object: ObjectIndex,
		data: T,
		timeout: Duration,
	) -> Result<(), sdo::SdoError> {
		use std::borrow::Borrow;
		let buffer = data.to_buffer();
		sdo::sdo_download(self, node_id, sdo, object, buffer.borrow(), timeout).await
	}

	/// Get the full PDO configuration of an RPDO of a remote node.
	pub async fn read_rpdo_configuration(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		pdo: u16,
		timeout: Duration,
	) -> Result<pdo::RpdoConfiguration, pdo::PdoConfigError>
	{
		pdo::read_rpdo_configuration(self, node_id, sdo, pdo, timeout).await
	}

	/// Get the full configuration of a TPDO of a remote node.
	pub async fn read_tpdo_configuration(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		pdo: u16,
		timeout: Duration,
	) -> Result<pdo::TpdoConfiguration, pdo::PdoConfigError>
	{
		pdo::read_tpdo_configuration(self, node_id, sdo, pdo, timeout).await
	}

	/// Configure an RPDO of a remote node.
	pub async fn configure_rpdo(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		pdo: u16,
		config: &pdo::RpdoConfiguration,
		timeout: Duration,
	) -> Result<(), pdo::PdoConfigError>
	{
		pdo::configure_rpdo(self, node_id, sdo, pdo, config, timeout).await
	}

	/// Configure a TPDO of a remote node.
	pub async fn configure_tpdo(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		pdo: u16,
		config: &pdo::TpdoConfiguration,
		timeout: Duration,
	) -> Result<(), pdo::PdoConfigError>
	{
		pdo::configure_tpdo(self, node_id, sdo, pdo, config, timeout).await
	}

	/// Enable or disable an RPDO of a remote node.
	pub async fn enable_rpdo(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		pdo: u16,
		enable: bool,
		timeout: Duration,
	) -> Result<(), pdo::PdoConfigError>
	{
		pdo::enable_rpdo(self, node_id, sdo, pdo, enable, timeout).await
	}

	/// Enable or disable a TPDO of a remote node.
	pub async fn enable_tpdo(
		&mut self,
		node_id: u8,
		sdo: sdo::SdoAddress,
		pdo: u16,
		enable: bool,
		timeout: Duration,
	) -> Result<(), pdo::PdoConfigError>
	{
		pdo::enable_tpdo(self, node_id, sdo, pdo, enable, timeout).await
	}

	/// Send a SYNC command to the CAN network.
	pub async fn send_sync(
		&mut self,
		counter: Option<NonZeroU8>,
	) -> Result<(), std::io::Error> {
		sync::send_sync(self, counter).await
	}

	/// Receive a new message from the CAN bus that that matches the given predicate.
	///
	/// Messages already in the read queue are not returned.
	/// If a message does not match the filter, it is added to the read queue.
	async fn recv_new_filtered<F>(
		&mut self,
		predicate: F,
		timeout: Duration,
	) -> std::io::Result<Option<CanFrame>>
	where
		F: FnMut(&CanFrame) -> bool,
	{
		let receive_loop = async move {
			let mut predicate = predicate;
			loop {
				let frame = self.socket.recv().await?;
				if predicate(&frame) {
					return Ok(frame);
				} else {
					// TODO: Save messages for later delivery?
					// self.read_queue.push(frame)
				}
			}
		};

		tokio::time::timeout(timeout, receive_loop)
			.await
			.ok()
			.transpose()
	}

	/// Receive a new message from the CAN bus that that matches the given function code and node ID.
	///
	/// RTR (request-to-read) messages are filtered out (not returned).
	///
	/// Messages already in the read queue are not returned.
	/// If a message does not match the filter, it is added to the read queue.
	async fn recv_new_by_can_id(&mut self, can_id: CanBaseId, timeout: Duration) -> std::io::Result<Option<CanFrame>> {
		self.recv_new_filtered(|frame| !frame.is_rtr() && frame.id().to_base().ok() == Some(can_id), timeout).await
	}
}

impl ObjectIndex {
	/// Create a new object index from a main index and a subindex.
	pub fn new(index: u16, subindex: u8) -> Self {
		Self { index, subindex }
	}
}

impl std::fmt::Debug for ObjectIndex {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ObjectIndex")
			.field("index", &format_args!("0x{:04X}", self.index))
			.field("subindex", &format_args!("0x{:02X}", self.subindex))
			.finish()
	}
}
