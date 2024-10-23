use assert2::{assert, let_assert};
use can_socket::{base_id, extended_id, id, CanId};

#[test]
fn id_macro_works() {
	let_assert!(CanId::Base(id) = id!(5));
	assert!(id.as_u16() == 5);

	let_assert!(CanId::Base(id) = id!(base: 5));
	assert!(id.as_u16() == 5);

	let_assert!(CanId::Extended(id) = id!(extended: 5));
	assert!(id.as_u32() == 5);

	let_assert!(CanId::Extended(id) = id!(0x10 << 16 | 0x50));
	assert!(id.as_u32() == 0x10_0050);
}

#[test]
fn base_id_macro_works() {
	let id = base_id!(5);
	assert!(id.as_u16() == 5);

	let id = base_id!(can_socket::MAX_CAN_ID_BASE);
	assert!(id.as_u16() == can_socket::MAX_CAN_ID_BASE);
}

#[test]
fn extended_id_macro_works() {
	let id = extended_id!(5);
	assert!(id.as_u32() == 5);
}
