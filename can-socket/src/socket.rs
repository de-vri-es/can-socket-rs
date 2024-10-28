use crate::{CanFilter, CanFrame, CanInterface};

/// A synchronous CAN socket.
///
/// Used to send and receive [`CanFrame`]'s over the network.
///
/// Although the socket is synchronous,
/// it can be put into non-blocking mode with [`Self::set_nonblocking()`].
#[repr(transparent)]
pub struct CanSocket {
	inner: crate::sys::Socket,
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
	pub fn bind(interface: impl AsRef<str>) -> std::io::Result<Self> {
		let inner = crate::sys::Socket::new(false)?;
		let interface = inner.get_interface_by_name(interface.as_ref())?;
		inner.bind(&interface)?;
		Ok(Self { inner })
	}

	/// Create a new socket bound to a interface by index.
	pub fn bind_interface_index(index: u32) -> std::io::Result<Self> {
		let inner = crate::sys::Socket::new(false)?;
		inner.bind(&CanInterface::from_index(index).inner)?;
		Ok(Self { inner })
	}

	/// Create a new socket bound to all CAN interfaces on the system.
	///
	/// You can use [`Self::recv_from()`] if you need to know on which interface a frame was received,
	/// and [`Self::send_to()`] to send a frame on a particular interface.
	pub fn bind_all() -> std::io::Result<Self> {
		Self::bind_interface_index(0)
	}

	/// Get the interface this socket is bound to.
	///
	/// If the socket is bound to all interfaces, the returned `CanInterface` will report index 0.
	pub fn local_addr(&self) -> std::io::Result<crate::CanInterface> {
		Ok(CanInterface {
			inner: self.inner.local_addr()?,
		})
	}

	/// Set the socket in non-blocking or blocking mode.
	///
	/// If the socket is set in non-blocking mode, send and receive operations will never block.
	/// Instead, if the operation can not be completed immediately, it will fail with a [`std::io::ErrorKind::WouldBlock`] error.
	pub fn set_nonblocking(&self, non_blocking: bool) -> std::io::Result<()> {
		self.inner.set_nonblocking(non_blocking)
	}

	/// Send a frame over the socket.
	///
	/// Note that if this function success, it only means that the kernel accepted the frame for transmission.
	/// It does not mean the frame has been successfully transmitted over the CAN bus.
	pub fn send(&self, frame: &CanFrame) -> std::io::Result<()> {
		self.inner.send(&frame.inner)
	}

	/// Send a frame over a particular interface.
	///
	/// Note that if this function success, it only means that the kernel accepted the frame for transmission.
	/// It does not mean the frame has been successfully transmitted over the CAN bus.
	pub fn send_to(&self, frame: &CanFrame, interface: &CanInterface) -> std::io::Result<()> {
		self.inner.send_to(&frame.inner, &interface.inner)
	}

	/// Receive a frame from the socket.
	pub fn recv(&self) -> std::io::Result<CanFrame> {
		Ok(CanFrame {
			inner: self.inner.recv()?,
		})
	}

	/// Receive a frame from the socket, including information about which interface the frame was received on.
	pub fn recv_from(&self) -> std::io::Result<(CanFrame, CanInterface)> {
		let (frame, interface) = self.inner.recv_from()?;
		let frame = CanFrame { inner: frame };
		let interface = CanInterface { inner: interface };
		Ok((frame, interface))
	}

	/// Set the list of filters on the socket.
	///
	/// When a socket is created, it will receive all frames from the CAN interface.
	/// You can restrict this by setting the filters with this function.
	///
	/// A frame has to match only one of the filters in the list to be received by the socket.
	pub fn set_filters(&self, filters: &[CanFilter]) -> std::io::Result<()> {
		self.inner.set_filters(filters)
	}

	/// Check if the loopback option of the socket is enabled.
	///
	/// When enabled (the default for new sockets),
	/// frames sent on the same interface by other sockets are also received by this socket.
	pub fn get_loopback(&self) -> std::io::Result<bool> {
		self.inner.get_loopback()
	}

	/// Enable or disabling the loopback option of the socket.
	///
	/// When enabled (the default for new sockets),
	/// frames sent on the same interface by other sockets are also received by this socket.
	///
	/// See `Self::set_receive_own_messages()` if you also want to receive messages sens on *this* socket.
	pub fn set_loopback(&self, enable: bool) -> std::io::Result<()> {
		self.inner.set_loopback(enable)
	}

	/// Check if the receive own messages option of the socket is enabled.
	///
	/// When this option is enabled, frames sent on this socket are also delivered to this socket.
	///
	/// Note that frames sent on this socket are subject to all the same filtering mechanisms as other frames.
	/// To receive frames send on this socket, you must also to ensure that the loopback option is enabled ([`Self::get_loopback()`]),
	/// and that the frame is not discarded by the filters ([`Self::set_filters()`]).
	pub fn get_receive_own_messages(&self) -> std::io::Result<bool> {
		self.inner.get_receive_own_messages()
	}

	/// Enable or disable the receive own messages option of the socket.
	///
	/// When this option is enabled, frames sent on this socket are also delivered to this socket.
	///
	/// Note that frames sent on this socket are subject to all the same filtering mechanisms as other frames.
	/// To receive frames send on this socket, you must also to ensure that the loopback option is enabled ([`Self::set_loopback()`]),
	/// and that the frame is not discarded by the filters ([`Self::set_filters()`]).
	pub fn set_receive_own_messages(&self, enable: bool) -> std::io::Result<()> {
		self.inner.set_receive_own_messages(enable)
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
			inner: crate::sys::Socket::from_raw_fd(fd)
		}
	}
}
