use tokio::io::unix::AsyncFd;

use crate::sys;
use crate::CanFilter;
use sys::CanFrame;
use sys::CanInterface;

pub struct CanSocket {
	io: AsyncFd<sys::Socket>,
}

impl std::fmt::Debug for CanSocket {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut debug = f.debug_struct("CanSocket");
		#[cfg(unix)]
		{
			use std::os::unix::io::AsRawFd;
			debug.field("fd", &self.as_raw_fd());
			debug.finish()
		}

		#[cfg(not(unix))]
		debug.finish_non_exhaustive()
	}
}

impl CanSocket {
	/// Create a new socket bound to a named CAN interface.
	///
	/// This function is not async as it will either succeed or fail immediately.
	pub fn bind(interface: impl AsRef<str>) -> std::io::Result<Self> {
		let inner = sys::Socket::new(true)?;
		let interface = inner.get_interface_by_name(interface.as_ref())?;
		inner.bind(&interface)?;
		let io = AsyncFd::new(inner)?;
		Ok(Self { io })
	}

	/// Create a new socket bound to a interface by index.
	///
	/// This function is not async as it will either succeed or fail immediately.
	pub fn bind_interface_index(index: u32) -> std::io::Result<Self> {
		let inner = sys::Socket::new(true)?;
		inner.bind(&CanInterface::from_index(index))?;
		let io = AsyncFd::new(inner)?;
		Ok(Self { io })
	}

	/// Create a new socket bound to all CAN interfaces on the system.
	///
	/// You can use [`Self::recv_from()`] if you need to know on which interface a frame was received,
	/// and [`Self::send_to()`] to send a frame on a particular interface.
	///
	/// This function is not async as it will either succeed or fail immediately.
	pub fn bind_all() -> std::io::Result<Self> {
		Self::bind_interface_index(0)
	}

	/// Send a frame over the socket.
	pub async fn send(&self, frame: &CanFrame) -> std::io::Result<()> {
		self.io.async_io(tokio::io::Interest::WRITABLE, |inner| {
			inner.send(frame)
		}).await
	}

	/// Send a frame over a particular interface.
	///
	/// The interface must match the interface the socket was bound to,
	/// or the socket must have been bound to all interfaces.
	pub async fn send_to(&self, frame: &CanFrame, interface: &CanInterface) -> std::io::Result<()> {
		self.io.async_io(tokio::io::Interest::WRITABLE, |inner| {
			inner.send_to(frame, interface)
		}).await
	}

	/// Receive a frame from the socket.
	pub async fn recv(&self) -> std::io::Result<CanFrame> {
		self.io.async_io(tokio::io::Interest::READABLE, |inner| {
			inner.recv()
		}).await
	}

	/// Receive a frame from the socket, including information about which interface the frame was received on.
	pub async fn recv_from(&self) -> std::io::Result<(CanFrame, CanInterface)> {
		self.io.async_io(tokio::io::Interest::READABLE, |inner| {
			inner.recv_from()
		}).await
	}

	/// Set the list of filters on the socket.
	///
	/// When a socket is created, it will receive all frames from the CAN interface.
	/// You can restrict this by setting the filters with this function.
	///
	/// A frame has to match only one of the filters in the list to be received by the socket.
	pub fn set_filters(&self, filters: &[CanFilter]) -> std::io::Result<()> {
		self.io.get_ref().set_filters(filters)
	}

	/// Check if the loopback option of the socket is enabled.
	///
	/// When enabled (the default for new sockets),
	/// frames sent on the same interface by other sockets are also received by this socket.
	pub fn get_loopback(&self) -> std::io::Result<bool> {
		self.io.get_ref().get_loopback()
	}

	/// Enable or disabling the loopback option of the socket.
	///
	/// When enabled (the default for new sockets),
	/// frames sent on the same interface by other sockets are also received by this socket.
	///
	/// See `Self::set_receive_own_messages()` if you also want to receive messages sens on *this* socket.
	pub fn set_loopback(&self, enable: bool) -> std::io::Result<()> {
		self.io.get_ref().set_loopback(enable)
	}

	/// Check if the receive own messages option of the socket is enabled.
	///
	/// When this option is enabled, frames sent on this socket are also delivered to this socket.
	///
	/// Note that frames sent on this socket are subject to all the same filtering mechanisms as other frames.
	/// To receive frames send on this socket, you must also to ensure that the loopback option is enabled ([`Self::get_loopback()`]),
	/// and that the frame is not discarded by the filters ([`Self::set_filters()`]).
	pub fn get_receive_own_messages(&self) -> std::io::Result<bool> {
		self.io.get_ref().get_receive_own_messages()
	}

	/// Enable or disable the receive own messages option of the socket.
	///
	/// When this option is enabled, frames sent on this socket are also delivered to this socket.
	///
	/// Note that frames sent on this socket are subject to all the same filtering mechanisms as other frames.
	/// To receive frames send on this socket, you must also to ensure that the loopback option is enabled ([`Self::set_loopback()`]),
	/// and that the frame is not discarded by the filters ([`Self::set_filters()`]).
	pub fn set_receive_own_messages(&self, enable: bool) -> std::io::Result<()> {
		self.io.get_ref().set_receive_own_messages(enable)
	}
}

impl std::os::fd::AsFd for CanSocket {
	fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
		self.io.as_fd()
	}
}

impl From<CanSocket> for std::os::fd::OwnedFd {
	fn from(value: CanSocket) -> Self {
		value.io.into_inner().into()
	}
}

impl TryFrom<std::os::fd::OwnedFd> for CanSocket {
	type Error = std::io::Error;

	fn try_from(value: std::os::fd::OwnedFd) -> std::io::Result<Self> {
		let io = AsyncFd::new(sys::Socket::from(value))?;
		Ok(Self { io })
	}
}

impl std::os::fd::AsRawFd for CanSocket {
	fn as_raw_fd(&self) -> std::os::fd::RawFd {
		self.io.as_raw_fd()
	}
}

impl std::os::fd::IntoRawFd for CanSocket {
	fn into_raw_fd(self) -> std::os::fd::RawFd {
		self.io.into_inner().into_raw_fd()
	}
}
