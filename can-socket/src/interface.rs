/// A CAN interface identified by its index.
///
/// This type is used as a socket address to bind CAN sockets to a specific interface.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct CanInterface {
	/// The inner `CanInterface`, directly usable as a socket address for a CAN socket.
	pub(crate) inner: crate::sys::CanInterface,
}

impl CanInterface {
	/// Create a new `CanInterface` from a raw index.
	///
	/// Index `0` represents the "bind all" interface.
	pub fn from_index(index: u32) -> Self {
		Self {
			inner: crate::sys::CanInterface::from_index(index),
		}
	}

	/// Resolve a CAN interface name to a [`CanInterface`].
	pub fn from_name(name: &str) -> std::io::Result<Self> {
		Ok(Self {
			inner: crate::sys::CanInterface::from_name(name)?,
		})
	}

	/// Get the index of the interface.
	pub fn index(&self) -> u32 {
		self.inner.index()
	}

	/// Look up the name of the interface from the index.
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
