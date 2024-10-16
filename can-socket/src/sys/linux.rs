use filedesc::FileDesc;
use std::ffi::{c_int, c_uint, c_void, CString};
use std::mem::MaybeUninit;

use crate::{CanBaseId, CanExtendedId, CanId};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
struct can_frame {
	pub can_id: u32,
	pub can_dlc: u8,
	_pad: u8,
	_res0: u8,
	pub len8_dlc: u8,
	pub data: [u8; 8],
}

#[derive(Debug)]
pub struct Socket {
	fd: FileDesc,
}

#[derive(Copy, Clone)]
pub struct CanFrame {
	inner: can_frame
}

#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CanInterface {
	index: u32,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct CanFilter {
	filter: libc::can_filter,
}

impl CanFrame {
	pub fn new(id: impl Into<CanId>, data: &[u8], data_length_code: Option<u8>) -> std::io::Result<Self> {
		let id = id.into();

		if data.len() > 8 {
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "maximum CAN data length is 8 bytes"));
		}
		if let Some(data_length_code) = data_length_code {
			if data.len() != 8 {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "data_length_code can only be set if data length is 8"));
			}
			if !(9..=15).contains(&data_length_code) {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "data_length_code must be in the range 9 to 15 (inclusive)"));
			}
		}

		let mut inner: can_frame = unsafe { std::mem::zeroed() };
		inner.can_id = match id {
			CanId::Extended(x) => x.as_u32() | libc::CAN_EFF_FLAG,
			CanId::Base(x) => x.as_u16().into(),
		};
		inner.can_dlc = data.len() as u8;
		inner.len8_dlc = data_length_code.unwrap_or(0);
		inner.data[..data.len()].copy_from_slice(data);
		Ok(Self { inner })
	}

	/// Create a new RTR (request-to-read) frame.
	pub fn new_rtr(id: impl Into<CanId>, data_len: u8) -> std::io::Result<Self> {
		let id = id.into();
		if data_len > 8 {
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "maximum CAN data length is 8 bytes"));
		}

		let mut inner: can_frame = unsafe { std::mem::zeroed() };
		inner.can_id = match id {
			CanId::Extended(x) => x.as_u32() | libc::CAN_EFF_FLAG,
			CanId::Base(x) => x.as_u16().into(),
		};
		inner.can_id |= libc::CAN_RTR_FLAG;
		inner.can_dlc = data_len;
		inner.len8_dlc = 0;
		Ok(Self { inner })
	}

	pub fn id(&self) -> CanId {
		// Unwrap should be fine: the kernel should never give us an invalid CAN ID,
		// and the Rust constructor doesn't allow it.
		if self.inner.can_id & libc::CAN_EFF_FLAG == 0 {
			CanId::new_base((self.inner.can_id & libc::CAN_SFF_MASK) as u16).unwrap()
		} else {
			CanId::new_extended(self.inner.can_id & libc::CAN_EFF_MASK).unwrap()
		}
	}

	pub fn is_rtr(&self) -> bool {
		self.inner.can_id & libc::CAN_RTR_FLAG != 0
	}

	pub fn data(&self) -> &[u8] {
		&self.inner.data[..self.inner.can_dlc as usize]
	}

	pub fn data_length_code(&self) -> Option<u8> {
		if self.inner.len8_dlc > 0 {
			Some(self.inner.len8_dlc)
		} else {
			None
		}
	}

	fn as_c_void_ptr(&self) -> *const c_void {
		(self as *const Self).cast()
	}
}

impl CanInterface {
	pub fn from_index(index: u32) -> Self {
		Self { index }
	}

	pub fn from_name(name: &str) -> std::io::Result<Self> {
		let name = CString::new(name)
			.map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "interface name contain a null byte"))?;
		let index = unsafe { libc::if_nametoindex(name.as_ptr()) };
		if index == 0 {
			return Err(std::io::Error::last_os_error());
		}
		Ok(Self::from_index(index))
	}

	pub fn index(&self) -> u32 {
		self.index
	}

	pub fn get_name(&self) -> std::io::Result<String> {
		let mut buffer = vec![0u8; libc::IF_NAMESIZE];
		let name = unsafe { libc::if_indextoname(self.index, buffer.as_mut_ptr().cast()) };
		if name.is_null() {
			return Err(std::io::Error::last_os_error());
		}
		if let Some(len) = buffer.iter().position(|&byte| byte == 0) {
			buffer.truncate(len)
		}
		String::from_utf8(buffer)
			.map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "interface name contains invalid UTF-8"))
	}

	fn to_address(&self) -> libc::sockaddr_can {
		unsafe {
			let mut addr: libc::sockaddr_can = std::mem::zeroed();
			addr.can_family = libc::AF_CAN as _;
			addr.can_ifindex = self.index as _;
			addr
		}
	}
}

impl Socket {
	pub fn new(non_blocking: bool) -> std::io::Result<Self> {
		let flags = match non_blocking {
			true => libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
			false => libc::SOCK_CLOEXEC,
		};
		unsafe {
			let fd = check_int(libc::socket(libc::PF_CAN, libc::SOCK_RAW | flags, libc::CAN_RAW))?;
			Ok(Self {
				fd: FileDesc::from_raw_fd(fd),
			})
		}
	}

	pub fn set_nonblocking(&self, non_blocking: bool) -> std::io::Result<()> {
		unsafe {
			let flags = check_int(libc::fcntl(self.fd.as_raw_fd(), libc::F_GETFL))?;
			let flags = match non_blocking {
				true => flags | libc::O_NONBLOCK,
				false => flags & !libc::O_NONBLOCK,
			};
			check_int(libc::fcntl(self.fd.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK))?;
		}
		Ok(())
	}

	pub fn get_interface_by_name(&self, name: &str) -> std::io::Result<CanInterface> {
		unsafe {
			let mut req: libc::ifreq = std::mem::zeroed();
			if name.len() + 1 > std::mem::size_of_val(&req.ifr_name) {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "interface name too long"));
			}
			if name.as_bytes().iter().any(|&byte| byte == 0) {
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "interface name contain a null byte"));
			}
			std::ptr::copy(name.as_ptr().cast(), req.ifr_name.as_mut_ptr(), name.len());
			check_int(libc::ioctl(self.fd.as_raw_fd(), libc::SIOCGIFINDEX as _, &mut req))?;
			Ok(CanInterface::from_index(req.ifr_ifru.ifru_ifindex as u32))
		}
	}

	pub fn bind(&self, interface: &CanInterface) -> std::io::Result<()> {
		unsafe {
			let addr = interface.to_address();
			check_int(libc::bind(
				self.fd.as_raw_fd(),
				&addr as *const _ as *const _,
				std::mem::size_of_val(&addr) as _),
			)?;
			Ok(())
		}
	}

	pub fn send(&self, frame: &CanFrame) -> std::io::Result<()> {
		unsafe {
			let written = check_isize(libc::send(
				self.fd.as_raw_fd(),
				frame.as_c_void_ptr(),
				std::mem::size_of_val(frame),
				0,
			))?;
			debug_assert!(written as usize == std::mem::size_of_val(frame));
			Ok(())
		}
	}

	pub fn send_to(&self, frame: &CanFrame, interface: &CanInterface) -> std::io::Result<()> {
		unsafe {
			let address = interface.to_address();
			let written = check_isize(libc::sendto(
				self.fd.as_raw_fd(),
				&frame as *const _ as *const _,
				std::mem::size_of_val(frame),
				0,
				&address as *const _ as *const _,
				std::mem::size_of_val(&address) as _,
			))?;
			debug_assert!(written as usize == std::mem::size_of_val(frame));
			Ok(())
		}
	}

	pub fn recv(&self) -> std::io::Result<CanFrame> {
		unsafe {
			let mut frame: MaybeUninit<CanFrame> = MaybeUninit::uninit();
			let read = check_isize(libc::recv(
				self.fd.as_raw_fd(),
				frame.as_mut_ptr().cast(),
				std::mem::size_of_val(&frame),
				0,
			))?;
			debug_assert!(read as usize == std::mem::size_of_val(&frame));
			Ok(frame.assume_init())
		}
	}

	pub fn recv_from(&self) -> std::io::Result<(CanFrame, CanInterface)> {
		unsafe {
			let mut frame: MaybeUninit<CanFrame> = MaybeUninit::uninit();
			let mut addr: libc::sockaddr_can = std::mem::zeroed();
			let read = check_isize(libc::recvfrom(
				self.fd.as_raw_fd(),
				frame.as_mut_ptr().cast(),
				std::mem::size_of_val(&frame),
				0,
				&mut addr as *mut _ as *mut _,
				std::mem::size_of_val(&addr) as _,
			))?;
			debug_assert!(read as usize == std::mem::size_of_val(&frame));

			Ok((frame.assume_init(), CanInterface { index: addr.can_ifindex as u32 }))
		}
	}

	pub fn set_filters(&self, filters: &[crate::CanFilter]) -> std::io::Result<()> {
		let len = std::mem::size_of_val(filters)
			.try_into()
			.map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "filter list too large"))?;
		unsafe {
			check_int(libc::setsockopt(
				self.fd.as_raw_fd(),
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_FILTER,
				filters.as_ptr().cast(),
				len,
			))?;
			Ok(())
		}
	}

	pub fn get_loopback(&self) -> std::io::Result<bool> {
		let mut enable: c_int = 0;
		let mut len: c_uint = std::mem::size_of::<c_int>() as c_uint;
		unsafe {
			check_int(libc::getsockopt(
				self.fd.as_raw_fd(),
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_LOOPBACK,
				&mut enable as *mut c_int as *mut c_void,
				&mut len,
			))?;
		}
		Ok(enable != 0)
	}

	pub fn set_loopback(&self, enable: bool) -> std::io::Result<()> {
		let enabled = c_int::from(enable);
		unsafe {
			check_int(libc::setsockopt(
				self.fd.as_raw_fd(),
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_LOOPBACK,
				&enabled as *const c_int as *const c_void,
				std::mem::size_of_val(&enabled) as u32,
			))?;
		}
		Ok(())
	}

	pub fn get_receive_own_messages(&self) -> std::io::Result<bool> {
		let mut enabled: c_int = 0;
		let mut len: c_uint = std::mem::size_of::<c_int>() as c_uint;
		unsafe {
			check_int(libc::getsockopt(
				self.fd.as_raw_fd(),
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_RECV_OWN_MSGS,
				&mut enabled as *mut c_int as *mut c_void,
				&mut len,
			))?;
		}
		Ok(enabled != 0)
	}

	pub fn set_receive_own_messages(&self, enable: bool) -> std::io::Result<()> {
		let enable = c_int::from(enable);
		unsafe {
			check_int(libc::setsockopt(
				self.fd.as_raw_fd(),
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_RECV_OWN_MSGS,
				&enable as *const c_int as *const c_void,
				std::mem::size_of_val(&enable) as u32,
			))?;
		}
		Ok(())
	}
}

impl CanFilter {
	pub const fn new_base(id: CanBaseId) -> Self {
		Self {
			filter: libc::can_filter {
				can_id: id.as_u16() as u32,
				can_mask: 0,
			},
		}
	}

	pub const fn new_extended(id: CanExtendedId) -> Self {
		Self {
			filter: libc::can_filter {
				can_id: id.as_u32(),
				can_mask: 0,
			},
		}
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_id_value(mut self) -> Self {
		self.filter.can_mask |= libc::CAN_EFF_MASK;
		self
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_id_mask(mut self, mask: u32) -> Self {
		self.filter.can_mask |= mask & libc::CAN_EFF_MASK;
		self
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_base_extended(mut self) -> Self {
		self.filter.can_mask |= libc::CAN_EFF_FLAG;
		self
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_exact_id(mut self) -> Self {
		self.filter.can_mask |= libc::CAN_EFF_MASK | libc::CAN_EFF_FLAG;
		self
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_rtr_only(mut self) -> Self {
		self.filter.can_id |= libc::CAN_RTR_FLAG;
		self.filter.can_mask |= libc::CAN_RTR_FLAG;
		self
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_data_only(mut self) -> Self {
		self.filter.can_id &= !libc::CAN_RTR_FLAG;
		self.filter.can_mask |= libc::CAN_RTR_FLAG;
		self
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn inverted(mut self, inverted: bool) -> Self {
		if inverted {
			self.filter.can_id |= libc::CAN_INV_FILTER;
		} else {
			self.filter.can_id &= !libc::CAN_INV_FILTER;
		}
		self
	}
}

fn check_int(return_value: c_int) -> std::io::Result<c_int> {
	if return_value == -1 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(return_value)
	}
}

fn check_isize(return_value: isize) -> std::io::Result<isize> {
	if return_value == -1 {
		Err(std::io::Error::last_os_error())
	} else {
		Ok(return_value)
	}
}

impl std::os::fd::AsFd for Socket {
	fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
		self.fd.as_fd()
	}
}

impl From<Socket> for std::os::fd::OwnedFd {
	fn from(value: Socket) -> Self {
		value.fd.into()
	}
}

impl From<std::os::fd::OwnedFd> for Socket {
	fn from(value: std::os::fd::OwnedFd) -> Self {
		Self {
			fd: FileDesc::from(value),
		}
	}
}

impl std::os::fd::AsRawFd for Socket {
	fn as_raw_fd(&self) -> std::os::fd::RawFd {
		self.fd.as_raw_fd()
	}
}

impl std::os::fd::IntoRawFd for Socket {
	fn into_raw_fd(self) -> std::os::fd::RawFd {
		self.fd.into_raw_fd()
	}
}

impl std::os::fd::FromRawFd for Socket {
	unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
		Self {
			fd: FileDesc::from_raw_fd(fd)
		}
	}
}
