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
	/// A frame has to match only one of the filters in the list to be received by the socket.
	pub fn set_filters(&self, filters: &[CanFilter]) -> std::io::Result<()> {
		self.io.get_ref().set_filters(filters)
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
