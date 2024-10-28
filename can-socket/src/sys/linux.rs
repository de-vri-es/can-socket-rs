use filedesc::FileDesc;
use std::ffi::{c_int, c_void, CString};
use std::mem::MaybeUninit;

use crate::{CanData, CanId, ExtendedId, StandardId};

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct can_frame {
	pub can_id: u32,
	pub can_dlc: u8,
	_pad: u8,
	_res0: u8,
	pub len8_dlc: u8,
	pub data: [u8; 8],
}

pub(crate) struct Socket {
	fd: FileDesc,
}

#[derive(Copy, Clone)]
pub(crate) struct CanFrame {
	inner: can_frame
}

#[repr(transparent)]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct CanInterface {
	index: u32,
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct CanFilter {
	filter: libc::can_filter,
}

impl CanFrame {
	pub fn new(id: impl Into<CanId>, data: &crate::CanData) -> Self {
		let id = id.into();

		let mut inner: can_frame = unsafe { std::mem::zeroed() };
		inner.can_id = match id {
			CanId::Extended(x) => x.as_u32() | libc::CAN_EFF_FLAG,
			CanId::Standard(x) => x.as_u16().into(),
		};
		inner.can_dlc = data.len() as u8;
		inner.data[..data.len()].copy_from_slice(data);
		Self { inner }
	}

	pub fn new_rtr(id: impl Into<CanId>) -> Self {
		let id = id.into();

		let mut inner: can_frame = unsafe { std::mem::zeroed() };
		inner.can_id = match id {
			CanId::Extended(x) => x.as_u32() | libc::CAN_EFF_FLAG,
			CanId::Standard(x) => x.as_u16().into(),
		};
		inner.can_id |= libc::CAN_RTR_FLAG;
		inner.can_dlc = 0;
		inner.len8_dlc = 0;
		Self { inner }
	}

	pub fn id(&self) -> CanId {
		// Unwrap should be fine: the kernel should never give us an invalid CAN ID,
		// and the Rust constructor doesn't allow it.
		if self.inner.can_id & libc::CAN_EFF_FLAG == 0 {
			CanId::new_standard((self.inner.can_id & libc::CAN_SFF_MASK) as u16).unwrap()
		} else {
			CanId::new_extended(self.inner.can_id & libc::CAN_EFF_MASK).unwrap()
		}
	}

	pub fn is_rtr(&self) -> bool {
		self.inner.can_id & libc::CAN_RTR_FLAG != 0
	}

	pub fn data(&self) -> Option<CanData> {
		if self.is_rtr() {
			None
		} else {
			Some(CanData {
				data: self.inner.data,
				len: self.inner.can_dlc,
			})
		}
	}

	pub fn set_data_length_code(&mut self, dlc: u8) -> Result<(), ()> {
		if dlc > 15 {
			return Err(());
		}

		self.inner.can_dlc = dlc.clamp(0, 8);
		if dlc > 8 {
			self.inner.len8_dlc = dlc;
		} else {
			self.inner.len8_dlc = 0;
		}

		self.inner.data[self.inner.can_dlc as usize..].fill(0);
		Ok(())
	}

	pub fn data_length_code(&self) -> u8 {
		if self.inner.can_dlc == 8 && self.inner.len8_dlc > 8 {
			self.inner.len8_dlc
		} else {
			self.inner.can_dlc
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

	pub fn local_addr(&self) -> std::io::Result<CanInterface> {
		unsafe {
			let mut addr: libc::sockaddr_can = std::mem::zeroed();
			let mut addr_len: libc::socklen_t = std::mem::size_of_val(&addr) as _;
			let _addr_len = check_int(libc::getsockname(
				self.fd.as_raw_fd(),
				&mut addr as *mut libc::sockaddr_can as *mut libc::sockaddr,
				&mut addr_len,
			));
			Ok(CanInterface {
				index: addr.can_ifindex as u32,
			})
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
		unsafe {
			set_socket_option_slice(
				&self.fd,
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_FILTER,
				filters,
			)?;
			Ok(())
		}
	}

	pub fn get_loopback(&self) -> std::io::Result<bool> {
		let enabled: c_int = unsafe {
			get_socket_option(
				&self.fd,
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_LOOPBACK,
			)?
		};
		Ok(enabled != 0)
	}

	pub fn set_loopback(&self, enable: bool) -> std::io::Result<()> {
		unsafe {
			set_socket_option(
				&self.fd,
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_LOOPBACK,
				&c_int::from(enable),
			)?;
		}
		Ok(())
	}

	pub fn get_receive_own_messages(&self) -> std::io::Result<bool> {
		let enabled: c_int = unsafe {
			get_socket_option(
				&self.fd,
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_RECV_OWN_MSGS,
			)?
		};
		Ok(enabled != 0)
	}

	pub fn set_receive_own_messages(&self, enable: bool) -> std::io::Result<()> {
		unsafe {
			set_socket_option(
				&self.fd,
				libc::SOL_CAN_RAW,
				libc::CAN_RAW_RECV_OWN_MSGS,
				&c_int::from(enable),
			)
		}
	}
}

impl CanFilter {
	pub const fn new_standard(id: StandardId) -> Self {
		Self {
			filter: libc::can_filter {
				can_id: id.as_u16() as u32,
				can_mask: 0,
			},
		}
	}

	pub const fn new_extended(id: ExtendedId) -> Self {
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

	pub const fn id(self) -> u32 {
		self.filter.can_id & libc::CAN_EFF_MASK
	}

	pub const fn id_mask(self) -> u32 {
		self.filter.can_mask & libc::CAN_EFF_MASK
	}

	pub const fn matches_rtr_frames(self) -> bool {
		let rtr_unmasked = self.filter.can_mask & libc::CAN_RTR_FLAG != 0;
		let is_rtr = self.filter.can_id & libc::CAN_RTR_FLAG != 0;
		!rtr_unmasked || is_rtr
	}

	pub const fn matches_data_frames(self) -> bool {
		let rtr_unmasked = self.filter.can_mask & libc::CAN_RTR_FLAG != 0;
		let is_rtr = self.filter.can_id & libc::CAN_RTR_FLAG != 0;
		!rtr_unmasked || !is_rtr
	}

	pub const fn matches_standard_frames(self) -> bool {
		let frame_type_unmasked = self.filter.can_mask & libc::CAN_EFF_FLAG != 0;
		let is_extended = self.filter.can_id & libc::CAN_EFF_FLAG != 0;
		!frame_type_unmasked || !is_extended
	}

	pub const fn matches_extended_frames(self) -> bool {
		let frame_type_unmasked = self.filter.can_mask & libc::CAN_EFF_FLAG != 0;
		let is_extended = self.filter.can_id & libc::CAN_EFF_FLAG != 0;
		!frame_type_unmasked || is_extended
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_id_mask(mut self, mask: u32) -> Self {
		self.filter.can_mask |= mask & libc::CAN_EFF_MASK;
		self
	}

	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_frame_format(mut self) -> Self {
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

	pub const fn is_inverted(self) -> bool {
		self.filter.can_id & libc::CAN_INV_FILTER != 0
	}

	pub const fn test(self, frame: &CanFrame) -> bool {
		let id = self.filter.can_id & !libc::CAN_INV_FILTER;
		let frame_matches = frame.inner.can_id & self.filter.can_mask == id & self.filter.can_mask;
		if self.is_inverted() {
			frame_matches
		} else {
			!frame_matches
		}
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

unsafe fn set_socket_option<T: Copy>(socket: &FileDesc, level: c_int, option: c_int, value: &T) -> std::io::Result<()> {
	let len = std::mem::size_of_val(value).try_into().map_err(|_| std::io::ErrorKind::InvalidInput)?;
	let value: *const T = value;
	check_int(libc::setsockopt(socket.as_raw_fd(), level, option, value.cast(), len))?;
	Ok(())
}

unsafe fn set_socket_option_slice<T: Copy>(socket: &FileDesc, level: c_int, option: c_int, value: &[T]) -> std::io::Result<()> {
	let len = std::mem::size_of_val(value).try_into().map_err(|_| std::io::ErrorKind::InvalidInput)?;
	let value = value.as_ptr();
	check_int(libc::setsockopt(socket.as_raw_fd(), level, option, value.cast(), len))?;
	Ok(())
}

unsafe fn get_socket_option<T: Copy + Default>(socket: &FileDesc, level: c_int, option: c_int) -> std::io::Result<T> {
	let mut value = T::default();
	let mut len = std::mem::size_of::<T>().try_into().unwrap();
	{
		let value: *mut T = &mut value;
		check_int(libc::getsockopt(socket.as_raw_fd(), level, option, value.cast(), &mut len))?;
	}
	Ok(value)
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
