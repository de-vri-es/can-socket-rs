use tokio::io::unix::AsyncFd;

use crate::sys;
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
	pub fn bind(interface: impl AsRef<str>) -> std::io::Result<Self> {
		let inner = sys::Socket::new(true)?;
		let interface = inner.get_interface_by_name(interface.as_ref())?;
		inner.bind(&interface)?;
		let io = AsyncFd::new(inner)?;
		Ok(Self { io })
	}

	pub fn bind_interface_index(index: u32) -> std::io::Result<Self> {
		let inner = sys::Socket::new(true)?;
		inner.bind(&CanInterface::from_index(index))?;
		let io = AsyncFd::new(inner)?;
		Ok(Self { io })
	}

	pub fn bind_all() -> std::io::Result<Self> {
		Self::bind_interface_index(0)
	}

	pub async fn send(&self, frame: &CanFrame) -> std::io::Result<()> {
		self.io.async_io(tokio::io::Interest::WRITABLE, |inner| {
			inner.send(frame)
		}).await
	}

	pub async fn send_to(&self, frame: &CanFrame, interface: &CanInterface) -> std::io::Result<()> {
		self.io.async_io(tokio::io::Interest::WRITABLE, |inner| {
			inner.send_to(frame, interface)
		}).await
	}

	pub async fn recv(&self) -> std::io::Result<CanFrame> {
		self.io.async_io(tokio::io::Interest::READABLE, |inner| {
			inner.recv()
		}).await
	}

	pub async fn recv_from(&self) -> std::io::Result<(CanFrame, CanInterface)> {
		self.io.async_io(tokio::io::Interest::READABLE, |inner| {
			inner.recv_from()
		}).await
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
