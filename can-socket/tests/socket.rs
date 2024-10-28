use std::path::Path;

use assert2::{assert, let_assert};
use can_socket::{CanData, CanFilter, CanFrame, CanSocket, ExtendedId, StandardId};

fn random_string(len: usize) -> String {
	use rand::Rng;
	use rand::distributions::Alphanumeric;

	let mut rng = rand::thread_rng();
	let mut string = String::with_capacity(len);
	for _ in 0..len {
		string.push(char::from(rng.sample(Alphanumeric)));
	}
	string
}

#[derive(Debug)]
struct TempInterface {
	name: String,
}

impl TempInterface {
	fn new() -> Result<Self, String> {
		let name = format!("vcan-{}", random_string(10));
		let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/create-vcan-interface");
		let output = std::process::Command::new(script)
			.arg("add")
			.arg(&name)
			.stdout(std::process::Stdio::piped())
			.stderr(std::process::Stdio::piped())
			.stdin(std::process::Stdio::null())
			.output()
			.map_err(|e| format!("failed to run `create-vcan-interface add`: {:?}", e.kind()))?;
		if output.status.success() {
			Ok(Self { name })
		} else {
			if let Ok(output) = std::str::from_utf8(&output.stdout) {
				let output = output.trim();
				if !output.is_empty() {
					println!("stdout of `create-vcan-interface add`:\n {output}\n");
				}
			}
			if let Ok(output) = std::str::from_utf8(&output.stderr) {
				let output = output.trim();
				if !output.is_empty() {
					return Err(output.into());
				}
			}
			Err(format!("ip link add: {:?}", output.status))
		}
	}

	fn remove(mut self) -> Result<(), String> {
		let name = std::mem::take(&mut self.name);
		if name.is_empty() {
			return Err("already removed".into());
		}

		let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/create-vcan-interface");
		let output = std::process::Command::new(script)
			.arg("del")
			.arg(&name)
			.stdout(std::process::Stdio::piped())
			.stderr(std::process::Stdio::piped())
			.stdin(std::process::Stdio::null())
			.output()
			.map_err(|e| format!("failed to run `create-vcan-interface del`: {:?}", e.kind()))?;
		if output.status.success() {
			Ok(())
		} else {
			if let Ok(output) = std::str::from_utf8(&output.stdout) {
				let output = output.trim();
				if !output.is_empty() {
					println!("stdout of `create-vcan-interface del`:\n {output}\n");
				}
			}
			if let Ok(output) = std::str::from_utf8(&output.stderr) {
				let output = output.trim();
				if !output.is_empty() {
					return Err(output.into());
				}
			}
			Err(format!("ip link add: {:?}", output.status))
		}
	}

	fn name(&self) -> &str {
		&self.name
	}
}

impl Drop for TempInterface {
	fn drop(&mut self) {
		if self.name.is_empty() {
			return;
		}
		let other = Self {
			name: std::mem::take(&mut self.name),
		};
		other.remove().unwrap()
	}
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn can_talk() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	let_assert!(Ok(socket_b) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));
	assert!(let Ok(()) = socket_b.set_nonblocking(true));

	assert!(let Ok(()) = socket_a.send(&CanFrame::new(1u8, [1, 2, 3])));
	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 1);
	assert!(frame.is_rtr() == false);
	assert!(frame.data() == Some(CanData::new([1, 2, 3])));
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn can_send_rtr() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	let_assert!(Ok(socket_b) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));
	assert!(let Ok(()) = socket_b.set_nonblocking(true));

	assert!(let Ok(()) = socket_a.send(&CanFrame::new_rtr(2u8)));
	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 2);
	assert!(frame.is_rtr() == true);
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn local_addr() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	let_assert!(Ok(local_addr) = socket_a.local_addr());
	assert!(local_addr.index() != 0);
	let_assert!(Ok(name) = local_addr.get_name(), "interface index: {}", local_addr.index());
	assert!(name == interface.name());
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn enable_recv_own_message() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));

	assert!(let Ok(true) = socket_a.get_loopback());
	assert!(let Ok(false) = socket_a.get_receive_own_messages());
	assert!(let Ok(()) = socket_a.set_receive_own_messages(true));
	assert!(let Ok(true) = socket_a.get_receive_own_messages());

	assert!(let Ok(()) = socket_a.send(&CanFrame::new(1u8, [1, 2, 3])));
	let_assert!(Ok(frame) = socket_a.recv());
	assert!(frame.id().as_u32() == 1);
	assert!(frame.is_rtr() == false);
	assert!(frame.data() == Some(CanData::new([1, 2, 3])));
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn disable_loopback() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));

	assert!(let Ok(true) = socket_a.get_loopback());
	assert!(let Ok(()) = socket_a.set_loopback(false));
	assert!(let Ok(false) = socket_a.get_loopback());

	assert!(let Ok(()) = socket_a.set_receive_own_messages(true));
	assert!(let Ok(false) = socket_a.get_loopback());
	assert!(let Ok(true) = socket_a.get_receive_own_messages());

	// It seems vcan pretends all frames from other sockets are non-local.
	// So we test by enabling `receive_own_messages` but disabling `loopback` and see if our own message gets dropped as expected.
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(1u8, [1, 2, 3])));
	let_assert!(Err(e) = socket_a.recv());
	assert!(e.kind() == std::io::ErrorKind::WouldBlock);
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn filter_exact_id() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	let_assert!(Ok(socket_b) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));
	assert!(let Ok(()) = socket_b.set_nonblocking(true));

	assert!(let Ok(()) = socket_b.set_filters(&[
		CanFilter::new(8u8.into()).match_exact_id()
	]));

	assert!(let Ok(()) = socket_a.send(&CanFrame::new(1u8, [1, 2, 3])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(8u8, [4, 5, 6])));
	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 8);
	assert!(frame.data() == Some(CanData::new([4, 5, 6])));

	let_assert!(Err(e) = socket_b.recv());
	assert!(e.kind() == std::io::ErrorKind::WouldBlock);
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn filter_exact_id_rtr_only() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	let_assert!(Ok(socket_b) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));
	assert!(let Ok(()) = socket_b.set_nonblocking(true));

	assert!(let Ok(()) = socket_b.set_filters(&[
		CanFilter::new(8u8.into()).match_exact_id().match_rtr_only(),
	]));

	assert!(let Ok(()) = socket_a.send(&CanFrame::new(1u8, [1, 2, 3])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(8u8, [4, 5, 6])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new_rtr(8u8)));
	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 8);
	assert!(frame.is_rtr());

	let_assert!(Err(e) = socket_b.recv());
	assert!(e.kind() == std::io::ErrorKind::WouldBlock);
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn filter_id_type() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	let_assert!(Ok(socket_b) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));
	assert!(let Ok(()) = socket_b.set_nonblocking(true));

	assert!(let Ok(()) = socket_b.set_filters(&[
		CanFilter::new_standard(0.into()).match_frame_format(),
	]));

	assert!(let Ok(()) = socket_a.send(&CanFrame::new(ExtendedId::from(5u8), [1])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(StandardId::from(6), [2])));
	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 6);
	assert!(frame.data() == Some(CanData::new([2])));

	let_assert!(Err(e) = socket_b.recv());
	assert!(e.kind() == std::io::ErrorKind::WouldBlock);
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn filter_mask() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	let_assert!(Ok(socket_b) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));
	assert!(let Ok(()) = socket_b.set_nonblocking(true));

	assert!(let Ok(()) = socket_b.set_filters(&[
		CanFilter::new_extended(0x1200u16.into()).match_id_mask(0xFFFFFF00),
	]));

	assert!(let Ok(()) = socket_a.send(&CanFrame::new(0x1300u16, [1])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(0x1200u16, [2])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(0x12FFu16, [3])));

	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 0x1200);
	assert!(frame.data() == Some(CanData::new([2])));

	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 0x12FF);
	assert!(frame.data() == Some(CanData::new([3])));

	let_assert!(Err(e) = socket_b.recv());
	assert!(e.kind() == std::io::ErrorKind::WouldBlock);
}

#[test]
#[cfg_attr(not(feature = "vcan-tests"), ignore = "enable the \"vcan-tests\" feature to enable this test")]
fn multiple_filters() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(Ok(socket_a) = CanSocket::bind(interface.name()));
	let_assert!(Ok(socket_b) = CanSocket::bind(interface.name()));
	assert!(let Ok(()) = socket_a.set_nonblocking(true));
	assert!(let Ok(()) = socket_b.set_nonblocking(true));

	assert!(let Ok(()) = socket_b.set_filters(&[
		CanFilter::new_extended(0x1200u16.into()).match_id_mask(0xFFFFFF00),
		CanFilter::new_extended(0x2000u16.into()).match_id_mask(0xFFFFFF00),
	]));

	assert!(let Ok(()) = socket_a.send(&CanFrame::new(0x1300u16, [1])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(0x1200u16, [2])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(0x12FFu16, [3])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(0x1900u16, [4])));
	assert!(let Ok(()) = socket_a.send(&CanFrame::new(0x2002u16, [5])));

	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 0x1200);
	assert!(frame.data() == Some(CanData::new([2])));

	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 0x12FF);
	assert!(frame.data() == Some(CanData::new([3])));

	let_assert!(Ok(frame) = socket_b.recv());
	assert!(frame.id().as_u32() == 0x2002);
	assert!(frame.data() == Some(CanData::new([5])));

	let_assert!(Err(e) = socket_b.recv());
	assert!(e.kind() == std::io::ErrorKind::WouldBlock);
}
