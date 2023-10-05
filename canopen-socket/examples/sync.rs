use can_socket::tokio::CanSocket;
use canopen_socket::CanOpenSocket;
use std::num::NonZeroU8;
use std::time::{Instant, Duration};

/// Send a SYNC command to the CANopen network.
#[derive(clap::Parser)]
struct Options {
	/// The CAN interface to use.
	interface: String,

	/// The counter value to add to the SYNC message.
	#[clap(long)]
	counter: Option<NonZeroU8>,

	/// Duration of the SYNC window in which we will wait for RPDOs to arrive.
	#[clap(long)]
	#[clap(value_parser = parse_duration)]
	#[clap(default_value = "0.010")]
	sync_window: Duration,
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

	log::info!("Sync window duration: {:?}", options.sync_window);
	let deadline = Instant::now() + options.sync_window;
	while let Some(frame) = socket.recv_frame_deadline(deadline).await {
		let frame = frame
			.map_err(|e| log::error!("Failed to receive CAN frame: {e}"))?;
		log::info!("Received CAN frame: {frame:#?}");
	}

	eprintln!("OK");
	Ok(())
}

fn parse_duration(input: &str) -> Result<Duration, &'static str> {
	let seconds: f64 = input.parse()
		.map_err(|_| "invalid duration: expected timeout in seconds")?;
	Ok(Duration::from_secs_f64(seconds))
}
