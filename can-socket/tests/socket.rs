use std::path::Path;

use assert2::{assert, let_assert};
use can_socket::CanSocket;

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
fn can_talk() {
	let_assert!(Ok(interface) = TempInterface::new());
	let_assert!(socket_a = CanSocket::bind(interface.name()));
	let_assert!(socket_b = CanSocket::bind(interface.name()));
}
