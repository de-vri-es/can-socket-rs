use can_socket::tokio::CanSocket;
use canopen_tokio::CanOpenSocket;
use canopen_tokio::ObjectIndex;
use canopen_tokio::sdo::SdoAddress;
use std::time::Duration;

#[derive(clap::Parser)]
struct Options {
	/// The CAN interface to use.
	interface: String,

	/// The node to write to.
	#[clap(value_parser(parse_number::<u8>))]
	node_id: u8,

	/// The object index to write to.
	#[clap(value_parser(parse_number::<u16>))]
	index: u16,

	/// The object subindex to write to.
	#[clap(value_parser(parse_number::<u8>))]
	subindex: u8,

	/// The data to write.
	#[clap(value_parser(parse_number::<u8>))]
	data: Vec<u8>,

	/// Timeout in seconds for receiving the reply.
	#[clap(long, short)]
	#[clap(value_parser(parse_timeout))]
	#[clap(default_value = "1")]
	timeout: Duration,
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
		.map_err(|e| log::error!("Failed to create CAN socket for interface {}: {e}", options.interface))?;
	let mut socket = CanOpenSocket::new(socket);

	let object = ObjectIndex::new(options.index, options.subindex);
	socket.sdo_download(options.node_id, SdoAddress::standard(), object, &options.data, options.timeout).await
		.map_err(|e| log::error!("{e}"))?;
	Ok(())
}

fn parse_timeout(input: &str) -> Result<Duration, &'static str> {
	let seconds: f64 = input.parse()
		.map_err(|_| "invalid duration: expected timeout in seconds")?;
	Ok(Duration::from_secs_f64(seconds))
}

fn parse_number<T>(input: &str) -> Result<T, String>
where
	T: TryFrom<i128>,
	T::Error: std::fmt::Display,
{
	let value = if let Some(hexadecimal) = input.strip_prefix("0x") {
		i128::from_str_radix(hexadecimal, 16)
			.map_err(|e| e.to_string())?
	} else if let Some(octal) = input.strip_prefix("0o") {
		i128::from_str_radix(octal, 8)
			.map_err(|e| e.to_string())?
	} else if let Some(binary) = input.strip_prefix("0b") {
		i128::from_str_radix(binary, 2)
			.map_err(|e| e.to_string())?
	} else {
		input.parse::<i128>()
			.map_err(|e| e.to_string())?
	};
	T::try_from(value)
		.map_err(|e| format!("value out of range: {e}"))
}

