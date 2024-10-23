use can_socket::tokio::CanSocket;
use canopen_tokio::CanOpenSocket;
use canopen_tokio::sdo::SdoAddress;
use std::time::Duration;

#[derive(clap::Parser)]
struct Options {
	/// The CAN interface to use.
	interface: String,

	/// The node read from.
	#[clap(value_parser(parse_number::<u8>))]
	node_id: u8,

	/// Configure the specified RPDO.
	#[clap(long)]
	#[clap(group = "pdo")]
	#[clap(value_parser(parse_number::<u16>))]
	rpdo: Option<u16>,

	/// Configure the specified TPDO.
	#[clap(long)]
	#[clap(group = "pdo")]
	#[clap(value_parser(parse_number::<u16>))]
	tpdo: Option<u16>,

	/// Timeout in seconds for individual SDO operations.
	#[clap(long, short)]
	#[clap(value_parser(parse_timeout))]
	#[clap(default_value = "1")]
	timeout: Duration,
}

#[derive(Clone, clap::ValueEnum)]
enum PdoType {
	Rpdo,
	Tpdo,
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

	if let Some(pdo) = options.rpdo {
		let config = socket.read_rpdo_configuration(options.node_id, SdoAddress::standard(), pdo, options.timeout).await
			.map_err(|e| log::error!("Failed to read configuration of RPDO {} of node {}: {e}", pdo, options.node_id))?;
		println!("{config:#?}");
	} else if let Some(pdo) = options.tpdo {
		let config = socket.read_tpdo_configuration(options.node_id, SdoAddress::standard(), pdo, options.timeout).await
			.map_err(|e| log::error!("Failed to read configuration of TPDO {} of node {}: {e}", pdo, options.node_id))?;
		println!("{config:#?}");
	}

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
