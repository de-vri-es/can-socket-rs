mod sys;

pub struct CanSocket {
	inner: sys::Socket,
}

pub use sys::CanFrame;
pub use sys::CanInterface;

impl CanSocket {
	pub fn bind(interface: impl AsRef<str>) -> std::io::Result<Self> {
		let inner = sys::Socket::new(false)?;
		let interface = inner.get_interface_by_name(interface.as_ref())?;
		inner.bind(&interface)?;
		Ok(Self { inner })
	}

	pub fn bind_interface_index(index: u32) -> std::io::Result<Self> {
		let inner = sys::Socket::new(false)?;
		inner.bind(&CanInterface::from_index(index))?;
		Ok(Self { inner })
	}

	pub fn bind_all() -> std::io::Result<Self> {
		Self::bind_interface_index(0)
	}

	pub fn set_nonblocking(&self, non_blocking: bool) -> std::io::Result<()> {
		self.inner.set_nonblocking(non_blocking)
	}

	pub fn send(&self, frame: &CanFrame) -> std::io::Result<()> {
		self.inner.send(frame)
	}

	pub fn send_to(&self, frame: &CanFrame, interface: &CanInterface) -> std::io::Result<()> {
		self.inner.send_to(frame, interface)
	}

	pub fn recv(&self) -> std::io::Result<CanFrame> {
		self.inner.recv()
	}

	pub fn recv_from(&self) -> std::io::Result<(CanFrame, CanInterface)> {
		self.inner.recv_from()
	}
}

impl std::os::fd::AsFd for CanSocket {
	fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
		self.inner.as_fd()
	}
}

impl From<CanSocket> for std::os::fd::OwnedFd {
	fn from(value: CanSocket) -> Self {
		value.inner.into()
	}
}

impl From<std::os::fd::OwnedFd> for CanSocket {
	fn from(value: std::os::fd::OwnedFd) -> Self {
		Self {
			inner: value.into(),
		}
	}
}

impl std::os::fd::AsRawFd for CanSocket {
	fn as_raw_fd(&self) -> std::os::fd::RawFd {
		self.inner.as_raw_fd()
	}
}

impl std::os::fd::IntoRawFd for CanSocket {
	fn into_raw_fd(self) -> std::os::fd::RawFd {
		self.inner.into_raw_fd()
	}
}

impl std::os::fd::FromRawFd for CanSocket {
	unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
		Self {
			inner: sys::Socket::from_raw_fd(fd)
		}
	}
}

impl std::fmt::Debug for CanFrame {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("CanFrame")
			.field("id", &format_args!("0x{:03X}", self.id()))
			.field("data", &format_args!("{:02X?}", self.data()))
			.field("data_length_code", &self.data_length_code())
			.finish()
	}
}
