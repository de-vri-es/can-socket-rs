use can_socket::tokio::CanSocket;

#[derive(clap::Parser)]
struct Options {
	/// The interface to read from.
	#[clap(long, short)]
	interface: String,

	/// Number of frames to read.
	///
	/// Will keep reading forever if you specify 0.
	#[clap(long, short)]
	count: usize,
}

#[tokio::main]
async fn main() {
	if let Err(()) = do_main(clap::Parser::parse()).await {
		std::process::exit(1);
	}
}

async fn do_main(options: Options) -> Result<(), ()> {
	let socket = CanSocket::bind(&options.interface)
		.map_err(|e| eprintln!("Failed to create CAN socket for interface {}: {e}", options.interface))?;

	for i in 0.. {
		if options.count != 0 && i == options.count {
			break;
		}
		eprintln!("Reading frame {i}...");
		let frame = socket.recv()
			.await
			.map_err(|e| eprintln!("Failed to receive frame on interface {}: {e}", options.interface))?;
		println!("{:#?}", frame);
		eprintln!()
	}

	Ok(())
}
