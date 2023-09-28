//! CANopen implementation for [`tokio`].

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use can_socket::tokio::CanSocket;
use can_socket::{CanFrame, CanBaseId};
use std::time::Duration;

mod id;
pub use id::CanBaseIdExt;

pub mod nmt;
pub mod sdo;

/// A CANopen socket.
///
/// Wrapper around a [`CanSocket`] that implements the `CANopen` protocol.
#[allow(missing_debug_implementations)]
pub struct CanOpenSocket {
	socket: CanSocket,
	read_queue: Vec<CanFrame>,
}

impl CanOpenSocket {
	/// Create a new CANopen socket from a [`CanSocket`].
	pub fn new(can_socket: CanSocket) -> Self {
		Self {
			socket: can_socket,
			read_queue: Vec::new(),
		}
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

	/// Read an object dictionary value by performing an "upload" from a SDO server.
	///
	/// Note that upload means "upload to server",
	/// so most people outside of [CiA](https://can-cia.org/) would call this a download.
	pub async fn read_sdo(
		&mut self,
		address: sdo::SdoAddress,
		node_id: u8,
		object_index: u16,
		object_subindex: u8,
		timeout: Duration,
	) -> Result<Vec<u8>, sdo::SdoError> {
		sdo::read_sdo(self, address, node_id, object_index, object_subindex, timeout).await
	}

	/// Write an object dictionary value by performing a "download" to a SDO server.
	///
	/// Note that download means "download to server",
	/// so most people outside of [CiA](https://can-cia.org/) would call this an upload.
	pub async fn write_sdo(
		&mut self,
		address: sdo::SdoAddress,
		node_id: u8,
		object_index: u16,
		object_subindex: u8,
		data: &[u8],
		timeout: Duration,
	) -> Result<(), sdo::SdoError> {
		sdo::write_sdo(self, address, node_id, object_index, object_subindex, data, timeout).await
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
					self.read_queue.push(frame)
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
	/// Messages already in the read queue are not returned.
	/// If a message does not match the filter, it is added to the read queue.
	async fn recv_new_by_can_id(&mut self, can_id: CanBaseId, timeout: Duration) -> std::io::Result<Option<CanFrame>> {
		self.recv_new_filtered(|frame| frame.id().to_base().ok() == Some(can_id), timeout).await
	}
}
