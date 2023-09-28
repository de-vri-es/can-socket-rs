use can_socket::tokio::CanSocket;
use canopen_socket::CanOpenSocket;
use std::num::NonZeroU8;

/// Send a SYNC command to the CANopen network.
#[derive(clap::Parser)]
struct Options {
	/// The CAN interface to use.
	interface: String,

	/// The counter value to add to the SYNC message.
	#[clap(long)]
	counter: Option<NonZeroU8>

}

#[tokio::main]
async fn main() {
	env_logger::builder()
		.filter_module(module_path!(), log::LevelFilter::Info)
		.parse_default_env()
		.init();
	if let Err(()) = do_main(clap::Parser::parse()).await {
		std::process::exit(1);
	}
}

async fn do_main(options: Options) -> Result<(), ()> {
	let socket = CanSocket::bind(&options.interface)
		.map_err(|e| eprintln!("Failed to create CAN socket for interface {}: {e}", options.interface))?;
	let mut socket = CanOpenSocket::new(socket);

	socket.send_sync(options.counter).await
		.map_err(|e| eprintln!("{e}"))?;

	eprintln!("OK");
	Ok(())
}
