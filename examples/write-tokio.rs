use can_socket::CanFrame;
use can_socket::tokio::CanSocket;

#[derive(clap::Parser)]
struct Options {
	/// The interface to read from.
	#[clap(long, short)]
	interface: String,

	/// The CAN ID of the message.
	#[clap(long)]
	id: u32,

	/// The data length code to send.
	///
	/// May only be specified for messages of size 8,
	/// and may only be a value from 9 to 15 (inclusive).
	#[clap(long)]
	data_length_code: Option<u8>,

	/// Number of times to send the frame.
	///
	/// Will keep sending forever if you specify 0.
	#[clap(long, short)]
	repeat: usize,

	/// Up to 8 data bytes to send.
	#[clap(num_args = 0..=8)]
	data: Vec<u8>,
}

#[tokio::main]
async fn main() {
	if let Err(()) = do_main(clap::Parser::parse()).await {
		std::process::exit(1);
	}
}

async fn do_main(options: Options) -> Result<(), ()> {
	let frame = CanFrame::new(options.id, &options.data, options.data_length_code)
		.map_err(|e| eprintln!("Invalid input: {e}"))?;

	let socket = CanSocket::bind(&options.interface)
		.map_err(|e| eprintln!("Failed to create CAN socket for interface {}: {e}", options.interface))?;

	for i in 0.. {
		if options.repeat != 0 && i == options.repeat {
			break;
		}
		eprintln!("Sending frame {i}...");
		socket.send(&frame)
			.await
			.map_err(|e| eprintln!("Failed to send frame on interface {}: {e}", options.interface))?;
		eprintln!()
	}

	Ok(())
}
