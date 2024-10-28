use crate::CanId;
use crate::error;

/// A CAN frame as transmitted over a CAN socket.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct CanFrame {
	pub(crate) inner: crate::sys::CanFrame,
}

impl CanFrame {
	/// Create a new data from with the given CAN ID and data payload.
	///
	/// To create a new data frame with a potentially invalid ID or data payload,
	/// use [`Self::try_new()`].
	#[inline]
	pub fn new(id: impl Into<CanId>, data: impl Into<CanData>) -> Self {
		Self {
			inner: crate::sys::CanFrame::new(id, &data.into())
		}
	}

	/// Create a new data from with the given CAN ID and data payload.
	///
	/// Will report an error if the ID or data is invalid.
	///
	/// You should normally prefer [`Self::new()`] if you can guarantee that the ID and data are valid.
	#[inline]
	pub fn try_new<Id, Data>(id: Id, data: Data) -> Result<Self, error::TryNewCanFrameError>
	where
		Id: TryInto<CanId>,
		error::TryNewCanFrameError: From<Id::Error>,
		Data: TryInto<CanData>,
		error::TryNewCanFrameError: From<Data::Error>,
	{
		Ok(Self::new(id.try_into()?, data.try_into()?))
	}

	/// Create a new remote transmission request (RTR) frame with a data length code of 0.
	///
	/// To set a different data length code, you can call [`Self::set_data_length_code()`]
	/// or [`Self::with_data_length_code()`] after constructing the RTR frame.
	#[inline]
	pub fn new_rtr(id: impl Into<CanId>) -> Self {
		Self {
			inner: crate::sys::CanFrame::new_rtr(id),
		}
	}

	/// Get the CAN ID of the frame.
	#[inline]
	pub fn id(&self) -> CanId {
		self.inner.id()
	}

	/// Check if this frame is a remote transmission request (an RTR frame).
	///
	/// RTR frames represent a request to transmit a value over the CAN bus.
	/// However, an application could decide to use RTR frames differently.
	///
	/// RTR frames have no associated data.
	#[inline]
	pub fn is_rtr(&self) -> bool {
		self.inner.is_rtr()
	}

	/// Get the data of the frame.
	///
	/// Returns `None` for RTR frames and `Some(data)` for data frames.
	#[inline]
	pub fn data(&self) -> Option<CanData> {
		self.inner.data()
	}

	/// Set the data length code of the frame.
	///
	/// If the data length code is higher than the current data length,
	/// the extra data bytes that become available will have a value of `0`.
	///
	/// If the data length code is in the range 9 to 15 (inclusive), the actual data length of the frame will be set to 8.
	/// However, if the CAN controller supports it, it may preserve the given data length code in the frame header.
	#[inline]
	pub fn set_data_length_code(&mut self, dlc: u8) -> Result<(), error::InvalidDataLengthCode> {
		self.inner.set_data_length_code(dlc)
			.map_err(|()| error::InvalidDataLengthCode { value: dlc })
	}

	/// Create a copy the frame with a modified data length code.
	///
	/// If the data length code is higher than the current data length,
	/// the extra data bytes that become available will have a value of `0`.
	///
	/// If the data length code is in the range 9 to 15 (inclusive), the actual data length of the frame will be set to 8.
	/// However, if the CAN controller supports it, it may preserve the given data length code in the frame header.
	#[inline]
	#[must_use = "this function returns a new frame, it does not modify self"]
	pub fn with_data_length_code(mut self, dlc: u8) -> Result<Self, error::InvalidDataLengthCode> {
		self.set_data_length_code(dlc)?;
		Ok(self)
	}

	/// Get the data length code of the frame (it may be higher than the number of data bytes in the frame).
	///
	/// If this is an RTR frame, it is often used to indicate how much bytes are expected in the response data frame.
	/// However, the application is free to use the data length code for a different purpose.
	///
	/// The CAN controller may preserve data length codes with a value above 8 (but at most 15).
	/// The data length should normally be assumed to be 8 bytes,
	/// and application is free to interpret the additional values according to it's own logic.
	/// Note that your CAN controller or driver may not preserve data length codes above `8`.
	#[inline]
	pub fn data_length_code(&self) -> u8 {
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

/// The data payload of a CAN frame.
///
/// Can hold up to 8 bytes.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CanData {
	pub(crate) data: [u8; 8],
	pub(crate) len: u8,
}

impl CanData {
	/// Construct a CAN data object from a supported fixed size array.
	///
	/// Also allows construction from any other type if it implements [`Into<CanData>`].
	pub fn new(data: impl Into<CanData>) -> Self {
		data.into()
	}

	/// Construct a CAN data object from a supported fixed size array.
	///
	/// Also allows construction from any other type if it implements [`Into<CanData>`].
	pub fn try_new<E>(data: impl TryInto<CanData, Error = E>) -> Result<Self, E> {
		data.try_into()
	}

	/// Get the data as a slice of bytes.
	#[inline]
	pub fn as_slice(&self) -> &[u8] {
		&self.data[..self.len.into()]
	}

	/// Get the data as a mutable slice of bytes.
	#[inline]
	pub fn as_slice_mut(&mut self) -> &mut [u8] {
		&mut self.data[..self.len.into()]
	}
}

impl std::fmt::Debug for CanData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Debug::fmt(self.as_slice(), f)
	}
}
impl std::ops::Deref for CanData {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl std::ops::DerefMut for CanData {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_slice_mut()
	}
}

impl std::borrow::Borrow<[u8]> for CanData {
	fn borrow(&self) -> &[u8] {
		self.as_slice()
	}
}

impl std::borrow::BorrowMut<[u8]> for CanData {
	fn borrow_mut(&mut self) -> &mut [u8] {
		self.as_slice_mut()
	}
}

impl AsRef<[u8]> for CanData {
	fn as_ref(&self) -> &[u8] {
		self.as_slice()
	}
}

impl AsMut<[u8]> for CanData {
	fn as_mut(&mut self) -> &mut [u8] {
		self.as_slice_mut()
	}
}

impl PartialEq<[u8]> for CanData {
	fn eq(&self, other: &[u8]) -> bool {
		self.as_slice() == other
	}
}

impl PartialEq<CanData> for [u8] {
	fn eq(&self, other: &CanData) -> bool {
		self == other.as_slice()
	}
}

macro_rules! impl_from_array {
	($n:literal) => {
		impl From<[u8; $n]> for CanData {
			fn from(value: [u8; $n]) -> Self {
				let mut data = [0; 8];
				data[..value.len()].copy_from_slice(&value);
				Self {
					data,
					len: $n,
				}
			}
		}

		impl<'a> From<&'a [u8; $n]> for CanData {
			fn from(value: &'a [u8; $n]) -> Self {
				let mut data = [0; 8];
				data[..value.len()].copy_from_slice(value);
				Self {
					data,
					len: $n,
				}
			}
		}

		impl TryFrom<CanData> for [u8; $n] {
			type Error = core::array::TryFromSliceError;

			fn try_from(other: CanData) -> Result<Self, Self::Error> {
				other.as_slice().try_into()
			}
		}

		impl<'a> TryFrom<&'a CanData> for [u8; $n] {
			type Error = core::array::TryFromSliceError;

			fn try_from(other: &'a CanData) -> Result<Self, Self::Error> {
				other.as_slice().try_into()
			}
		}

		impl PartialEq<[u8; $n]> for CanData {
			fn eq(&self, other: &[u8; $n]) -> bool {
				if self.len == $n {
					&self.data[..$n] == other
				} else {
					false
				}
			}
		}

		impl PartialEq<CanData> for [u8; $n] {
			fn eq(&self, other: &CanData) -> bool {
				other == self
			}
		}
	}
}

impl_from_array!(0);
impl_from_array!(1);
impl_from_array!(2);
impl_from_array!(3);
impl_from_array!(4);
impl_from_array!(5);
impl_from_array!(6);
impl_from_array!(7);
impl_from_array!(8);

impl TryFrom<&[u8]> for CanData {
	type Error = error::TryIntoCanDataError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() > 8 {
			Err(error::TryIntoCanDataError {
				len: value.len(),
			})
		} else {
			let mut data = [0; 8];
			data[..value.len()].copy_from_slice(value);
			Ok(Self {
				data,
				len: value.len() as u8,
			})
		}
	}
}

impl TryFrom<&Vec<u8>> for CanData {
	type Error = error::TryIntoCanDataError;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		value.as_slice().try_into()
	}
}

impl TryFrom<&Box<[u8]>> for CanData {
	type Error = error::TryIntoCanDataError;

	fn try_from(value: &Box<[u8]>) -> Result<Self, Self::Error> {
		let value: &[u8] = value;
		value.try_into()
	}
}


#[cfg(test)]
mod test {
	use super::*;
	use assert2::assert;
	use crate::can_id;

	#[test]
	fn can_frame_is_copy() {
		let frame = CanFrame::new(1u8, [1, 2, 3, 4]);
		let copy = frame;
		assert!(copy.id() == can_id!(1));
		assert!(copy.data() == Some(CanData::new([1, 2, 3, 4])));
	}

	#[test]
	fn can_data_from_array() {
		assert!(CanData::from([1]) == [1]);
		assert!(CanData::from([1, 2]) == [1, 2]);
		assert!(CanData::from([1, 2, 3]) == [1, 2, 3]);
		assert!(CanData::from([1, 2, 3, 4, 5]) == [1, 2, 3, 4, 5]);
		assert!(CanData::from([1, 2, 3, 4, 5, 6]) == [1, 2, 3, 4, 5, 6]);
		assert!(CanData::from([1, 2, 3, 4, 5, 6, 7]) == [1, 2, 3, 4, 5, 6, 7]);
		assert!(CanData::from([1, 2, 3, 4, 5, 6, 7, 8]) == [1, 2, 3, 4, 5, 6, 7, 8]);

		assert!(CanData::from([1, 2]) != [1]);
		assert!(CanData::from([1]) != [1, 2]);
	}
}
