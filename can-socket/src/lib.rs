pub mod error;

#[cfg(feature = "tokio")]
pub mod tokio;

mod can_id;
pub use can_id::{CanBaseId, CanExtendedId, CanId, MAX_CAN_ID_BASE, MAX_CAN_ID_EXTENDED};

mod filter;
pub use filter::CanFilter;

mod frame;
pub use frame::CanFrame;

mod interface;
pub use interface::CanInterface;

mod socket;
pub use socket::CanSocket;

mod sys;

/// Trait for types that can be used as a timeout or deadline.
pub trait Deadline {
	/// Get the instant at which the timeout/deadline expires.
	fn deadline(&self) -> std::time::Instant;
}

impl Deadline for std::time::Duration {
	/// Get the instant at which the timeout/deadline expires.
	///
	/// If the `"tokio"` feature is enabled, this uses [`tokio::time::Instant`] to compute the deadline.
	/// This means that [`tokio::time::pause()`] and [`tokio::time::advance()`] will work as expected.
	fn deadline(&self) -> std::time::Instant {
		// Use tokio's `Instant::now()` when the `tokio` feature is enabled.
		// This ensures that tokio::time::pause() and tokio::time::advance() will work.
		#[cfg(feature = "tokio")]
		{
			(::tokio::time::Instant::now() + *self).into_std()
		}

		#[cfg(not(feature = "tokio"))]
		{
			std::time::Instant::now() + *self
		}
	}
}

impl Deadline for std::time::Instant {
	fn deadline(&self) -> std::time::Instant {
		*self
	}
}

#[cfg(feature = "tokio")]
impl Deadline for ::tokio::time::Instant {
	fn deadline(&self) -> std::time::Instant {
		self.into_std()
	}
}
