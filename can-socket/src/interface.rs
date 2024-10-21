#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct CanInterface {
	pub(crate) inner: crate::sys::CanInterface,
}

impl CanInterface {
	pub fn from_index(index: u32) -> Self {
		Self {
			inner: crate::sys::CanInterface::from_index(index),
		}
	}

	pub fn from_name(name: &str) -> std::io::Result<Self> {
		Ok(Self {
			inner: crate::sys::CanInterface::from_name(name)?,
		})
	}

	pub fn index(&self) -> u32 {
		self.inner.index()
	}

	pub fn get_name(&self) -> std::io::Result<String> {
		self.inner.get_name()
	}
}

impl std::fmt::Debug for CanInterface {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let name = self.get_name();
		let mut debug = f.debug_struct("CanInterface");
		debug.field("index", &self.index());
		if let Ok(name) = &name {
			debug.field("name", name);
		}
		debug.finish()
	}
}
