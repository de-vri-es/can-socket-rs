use crate::{sys, StandardId, ExtendedId, CanFrame, CanId};

/// A CAN filter.
///
/// Can be used to have the kernel filter incoming frames before they are delivered to userspace,
/// to avoid unnecessarily waking up just to ignore a frame.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct CanFilter {
	filter: crate::sys::CanFilter,
}

impl CanFilter {
	/// Create a new pass-all CAN filter with a standard or extended ID.
	///
	/// The mask is still set to zero when the filter is created.
	/// You have to call additional methods to restrict what frames match filter.
	#[inline]
	pub const fn new(id: CanId) -> Self {
		match id {
			CanId::Standard(id) => Self::new_standard(id),
			CanId::Extended(id) => Self::new_extended(id),
		}
	}

	/// Create a new pass-all CAN filter with a standard ID.
	///
	/// The mask is still set to zero when the filter is created.
	/// You have to call additional methods to restrict what frames match filter.
	#[inline]
	pub const fn new_standard(id: StandardId) -> Self {
		Self {
			filter: sys::CanFilter::new_standard(id),
		}
	}

	/// Create a new pass-all CAN filter with an extended ID.
	///
	/// The mask is still set to zero when the filter is created.
	/// You have to call additional methods to restrict what frames match filter.
	#[inline]
	pub const fn new_extended(id: ExtendedId) -> Self {
		Self {
			filter: sys::CanFilter::new_extended(id),
		}
	}

	/// Restrict the filter to match only frames with the same numerical ID.
	///
	/// The filter will still accept extended and standard frames (if the numerical value is possible for standard frames).
	///
	/// Adds to any restrictions already applied to the filter.
	#[inline]
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_id_value(mut self) -> Self {
		self.filter = self.filter.match_id_value();
		self
	}

	/// Restrict the filter to match only frames where `frame.id & mask == filter.id & mask`.
	///
	/// Only the lower 29 bits of the mask are used.
	#[inline]
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_id_mask(mut self, mask: u32) -> Self {
		self.filter = self.filter.match_id_mask(mask);
		self
	}

	/// Restrict the filter to match only extended IDs or standard IDs (depending on the ID the filter was constructed with).
	///
	/// Adds to any restrictions already applied to the filter.
	#[inline]
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_frame_format(mut self) -> Self {
		self.filter = self.filter.match_frame_format();
		self
	}

	/// Restrict the filter to match only on exact ID matches.
	///
	/// This means the ID must match exactly, including the fact if it was an extended or standard ID.
	///
	/// This is equivalent to:
	/// ```
	/// # use can_socket::CanFilter;
	/// # fn foo(filter: CanFilter) -> CanFilter {
	/// filter.match_id_value().match_frame_format()
	/// # }
	/// ```
	///
	/// Adds to any restrictions already applied to the filter.
	#[inline]
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_exact_id(mut self) -> Self {
		self.filter = self.filter.match_exact_id();
		self
	}

	/// Restrict the filter to match only RTR frames.
	#[inline]
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_rtr_only(mut self) -> Self {
		self.filter = self.filter.match_rtr_only();
		self
	}

	/// Restrict the filter to match only data frames.
	///
	/// Overrides any previous calls to `Self::match_rtr_only()`.
	#[inline]
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_data_only(mut self) -> Self {
		self.filter = self.filter.match_data_only();
		self
	}

	/// Make the filter inverted or non-inverted.
	///
	/// When inverted, only frame that normally would not match the filter will match the filter.
	#[inline]
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn inverted(mut self, inverted: bool) -> Self {
		self.filter = self.filter.inverted(inverted);
		self
	}

	/// Check if the filter is inverted.
	///
	/// When inverted, only frame that normally would not match the filter will match the filter.
	#[inline]
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn is_inverted(self) -> bool {
		self.filter.is_inverted()
	}

	/// Test if a frame matches the filter.
	#[inline]
	pub const fn test(&self, frame: &CanFrame) -> bool {
		self.filter.test(&frame.inner)
	}
}

impl std::fmt::Debug for CanFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("CanFilter")
			.field("id", &format_args!("{:02X}", self.filter.id()))
			.field("mask", &format_args!("{:02X}", self.filter.id_mask()))
			.field("extended_frames", &self.filter.matches_extended_frames())
			.field("standard_frames", &self.filter.matches_standard_frames())
			.field("data_frames", &self.filter.matches_data_frames())
			.field("rtr_frames", &self.filter.matches_rtr_frames())
			.field("inverted", &self.filter.is_inverted())
			.finish()
	}
}
