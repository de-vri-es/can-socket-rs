use assert2::{assert, let_assert};
use can_socket::{standard_id, extended_id, can_id, CanId};

#[test]
fn id_macro_works() {
	let_assert!(CanId::Standard(id) = can_id!(5));
	assert!(id.as_u16() == 5);

	let_assert!(CanId::Standard(id) = can_id!(standard: 5));
	assert!(id.as_u16() == 5);

	let_assert!(CanId::Extended(id) = can_id!(extended: 5));
	assert!(id.as_u32() == 5);

	let_assert!(CanId::Extended(id) = can_id!(0x10 << 16 | 0x50));
	assert!(id.as_u32() == 0x10_0050);
}

#[test]
fn standard_id_macro_works() {
	let id = standard_id!(5);
	assert!(id.as_u16() == 5);

	let id = standard_id!(can_socket::MAX_STANDARD_ID);
	assert!(id.as_u16() == can_socket::MAX_STANDARD_ID);
}

#[test]
fn extended_id_macro_works() {
	let id = extended_id!(5);
	assert!(id.as_u32() == 5);
}
