use crate::{sys, CanBaseId, CanExtendedId, CanId};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct CanFilter {
	filter: crate::sys::CanFilter,
}

impl CanFilter {
	/// Create a new pass-all CAN filter with a base or extended ID.
	///
	/// The mask is still set to zero when the filter is created.
	/// You have to call additional methods to restrict what frames match filter.
	pub const fn new(id: CanId) -> Self {
		match id {
			CanId::Base(id) => Self::new_base(id),
			CanId::Extended(id) => Self::new_extended(id),
		}
	}

	/// Create a new pass-all CAN filter with a base ID.
	///
	/// The mask is still set to zero when the filter is created.
	/// You have to call additional methods to restrict what frames match filter.
	pub const fn new_base(id: CanBaseId) -> Self {
		Self {
			filter: sys::CanFilter::new_base(id),
		}
	}

	/// Create a new pass-all CAN filter with an extended ID.
	///
	/// The mask is still set to zero when the filter is created.
	/// You have to call additional methods to restrict what frames match filter.
	pub const fn new_extended(id: CanExtendedId) -> Self {
		Self {
			filter: sys::CanFilter::new_extended(id),
		}
	}

	/// Restrict the filter to match only frames with the same numerical ID.
	///
	/// The filter will still accept extended and base frames (if the numerical value is possible for base frames).
	///
	/// Adds to any restrictions already applied to the filter.
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_id_value(mut self) -> Self {
		self.filter = self.filter.match_id_value();
		self
	}

	/// Restrict the filter to match only frames where `frame.id & mask == filter.id & mask`.
	///
	/// Only the lower 29 bits of the mask are used.
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_id_mask(mut self, mask: u32) -> Self {
		self.filter = self.filter.match_id_mask(mask);
		self
	}

	/// Restrict the filter to match only extended IDs or base IDs (depending on the ID the filter was constructed with).
	///
	/// Adds to any restrictions already applied to the filter.
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_base_extended(mut self) -> Self {
		self.filter = self.filter.match_base_extended();
		self
	}

	/// Restrict the filter to match only on exact ID matches.
	///
	/// This means the ID must match exactly, including the fact if it was an extended or base ID.
	///
	/// This is equivalent to:
	/// ```
	/// # use can_socket::CanFilter;
	/// # fn foo(filter: CanFilter) -> CanFilter {
	/// filter.match_id_value().match_base_extended()
	/// # }
	/// ```
	///
	/// Adds to any restrictions already applied to the filter.
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_exact_id(mut self) -> Self {
		self.filter = self.filter.match_exact_id();
		self
	}

	/// Restrict the filter to match only RTR frames.
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_rtr_only(mut self) -> Self {
		self.filter = self.filter.match_rtr_only();
		self
	}

	/// Restrict the filter to match only data frames.
	///
	/// Overrides any previous calls to `Self::match_rtr_only()`.
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn match_data_only(mut self) -> Self {
		self.filter = self.filter.match_data_only();
		self
	}

	/// Make the filter inverted or non-inverted.
	///
	/// When inverted, only frame that normally would not match the filter will match the filter.
	#[must_use = "returns a new filter, does not modify the existing filter"]
	pub const fn inverted(mut self, inverted: bool) -> Self {
		self.filter = self.filter.inverted(inverted);
		self
	}
}
