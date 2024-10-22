use crate::CanId;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct CanFrame {
	pub(crate) inner: crate::sys::CanFrame,
}

impl CanFrame {
	pub fn new(id: impl Into<CanId>, data: &[u8], data_length_code: Option<u8>) -> std::io::Result<Self> {
		Ok(Self {
			inner: crate::sys::CanFrame::new(id, data, data_length_code)?,
		})
	}
	pub fn new_rtr(id: impl Into<CanId>, data_len: u8) -> std::io::Result<Self> {
		Ok(Self {
			inner: crate::sys::CanFrame::new_rtr(id, data_len)?,
		})
	}

	pub fn id(&self) -> CanId {
		self.inner.id()
	}

	pub fn is_rtr(&self) -> bool {
		self.inner.is_rtr()
	}

	pub fn data(&self) -> &[u8] {
		self.inner.data()
	}

	pub fn data_length_code(&self) -> Option<u8> {
		self.inner.data_length_code()
	}
}

impl std::fmt::Debug for CanFrame {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut debug = f.debug_struct("CanFrame");
		debug
			.field("id", &format_args!("{:?}", self.id()))
			.field("is_rtr", &self.is_rtr())
			.field("data_length_code", &self.data_length_code());
		if !self.is_rtr() {
			debug.field("data", &format_args!("{:02X?}", self.data()));
		}
		debug.finish()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use assert2::{assert, let_assert};

	#[test]
	fn can_frame_is_copy() {
		let_assert!(Ok(frame) = CanFrame::new(1u8, &[1, 2, 3, 4], None));
		let copy = frame;
		assert!(copy.id() == CanId::Base(1.into()));
		assert!(copy.data() == &[1, 2, 3, 4]);
	}
}
