//! CANopen implementation for [`tokio`].

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use can_socket::tokio::CanSocket;
use can_socket::{CanFrame, CanId, CanBaseId};
use std::time::Duration;

mod id;
pub use id::CanBaseIdExt;

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
		F: FnMut(CanBaseId) -> bool,
	{
		let receive_loop = async move {
			let mut predicate = predicate;
			loop {
				let frame = self.socket.recv().await?;
				match frame.id() {
					CanId::Base(id) if predicate(id) => {
						return Ok(frame)
					},
					_ => {
						self.read_queue.push(frame)
					}
				}
			}
		};

		tokio::time::timeout(timeout, receive_loop)
			.await
			.ok()
			.transpose()
	}

	/// Receive a new message from the CAN bus that that matches the given function code.
	///
	/// Messages already in the read queue are not returned.
	/// If a message does not match the filter, it is added to the read queue.
	async fn recv_new_by_funtion(&mut self, function_code: u16, timeout: Duration) -> std::io::Result<Option<CanFrame>> {
		self.recv_new_filtered(|id| id.function_code() == function_code, timeout).await
	}

	/// Receive a new message from the CAN bus that that matches the given function code and node ID.
	///
	/// Messages already in the read queue are not returned.
	/// If a message does not match the filter, it is added to the read queue.
	async fn recv_new_by_can_id(&mut self, can_id: CanBaseId, timeout: Duration) -> std::io::Result<Option<CanFrame>> {
		self.recv_new_filtered(|id| id == can_id, timeout).await
	}
}


// Having the submodules here looks weird, but we want the main `impl` block above to be the first in the documentation.
pub mod nmt;
pub mod sdo;
