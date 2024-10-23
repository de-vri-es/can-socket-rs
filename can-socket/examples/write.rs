use can_socket::{CanFrame, CanId, CanSocket};

#[derive(clap::Parser)]
struct Options {
	/// The interface to read from.
	#[clap(long, short)]
	interface: String,

	/// The CAN ID of the message.
	#[clap(long)]
	id: CanId,

	/// Send a RTR message instead of a data message.
	///
	/// The argument to the option is the number of bytes to request.
	#[clap(long)]
	rtr: Option<u8>,

	/// Number of times to send the frame.
	///
	/// Will keep sending forever if you specify 0.
	#[clap(long, short)]
	#[clap(default_value = "1")]
	repeat: usize,

	/// Up to 8 data bytes to send.
	#[clap(num_args = 0..=8)]
	#[clap(conflicts_with = "rtr")]
	data: Vec<u8>,
}

fn main() {
	if let Err(()) = do_main(clap::Parser::parse()) {
		std::process::exit(1);
	}
}

fn do_main(options: Options) -> Result<(), ()> {
	let frame = if let Some(len) = options.rtr {
		CanFrame::new_rtr(options.id)
			.with_data_length_code(len)
			.map_err(|e| eprintln!("Invalid input: {e}"))?
	} else {
		CanFrame::try_new(options.id, &options.data)
			.map_err(|e| eprintln!("Invalid input: {e}"))?
	};

	let socket = CanSocket::bind(&options.interface)
		.map_err(|e| eprintln!("Failed to create CAN socket for interface {}: {e}", options.interface))?;

	for i in 0.. {
		if options.repeat != 0 && i == options.repeat {
			break;
		}
		eprintln!("Sending frame {i}...");
		socket.send(&frame)
			.map_err(|e| eprintln!("Failed to send frame on interface {}: {e}", options.interface))?;
		eprintln!()
	}

	Ok(())
}
