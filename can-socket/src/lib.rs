//! CAN socket
//!
//! This library exposes a [`CanSocket`] and related types,
//! allowing you to communicate over a Controller Area Network (CAN) bus.
//!
//! The is a standard blocking or non-blocking [`CanSocket`],
//! and an asynchronous [`tokio::CanSocket`].
//!
//! This library uses the `SocketCAN` interface and only works on Linux.
//!
//! Supported features:
//! * Bind sockets to specific interfaces by name or index.
//! * Bind sockets to *all* CAN interfaces at the same time.
//! * Send and receive data frames and RTR frames.
//! * Send and receive standard frames and extended frames.
//! * Setting per-socket filters.
//! * Control over the `loopback` and `recv_own_msgs` options.
//! * Constructing compile-time checked CAN IDs.

#![cfg_attr(feature = "doc-cfg", feature(doc_cfg))]

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod error;

#[cfg(feature = "tokio")]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "tokio")))]
pub mod tokio;

mod id;
pub use id::{ExtendedId, CanId, StandardId, MAX_EXTENDED_ID, MAX_STANDARD_ID};

mod filter;
pub use filter::CanFilter;

mod frame;
pub use frame::{CanFrame, CanData};

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
	/// If the `"tokio"` feature is enabled, this uses [`tokio::time::Instant`][::tokio::time::Instant] to compute the deadline.
	/// This means that [`tokio::time::pause()`][::tokio::time::pause] and [`tokio::time::advance()`][::tokio::time::advance] will work as expected.
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
