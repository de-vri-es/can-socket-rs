use can_socket::tokio::CanSocket;
use canopen_socket::CanOpenSocket;
use canopen_socket::ObjectIndex;
use canopen_socket::sdo::SdoAddress;
use std::time::Duration;

#[derive(clap::Parser)]
struct Options {
	/// The CAN interface to use.
	interface: String,

	/// The node read from.
	#[clap(value_parser(parse_number::<u8>))]
	node_id: u8,

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
	let node_id = options.node_id;
	let timeout = options.timeout;
	let sdo = SdoAddress::standard();

	socket.send_nmt_command(node_id, canopen_socket::nmt::NmtCommand::ResetCommunication, timeout).await
		.map_err(|e| log::error!("Failed to reset communication of node {node_id}: {e}"))?;
	socket.send_nmt_command(node_id, canopen_socket::nmt::NmtCommand::Start, timeout).await
		.map_err(|e| log::error!("Failed to start node {node_id}: {e}"))?;
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6042, 0), 0, timeout).await
		.map_err(|e| log::error!("Failed to set target speed of node {node_id}: {e}"))?;
	socket.sdo_download::<u8>(node_id, sdo, ObjectIndex::new(0x6060, 0), 2, timeout).await
		.map_err(|e| log::error!("Failed to set operation mode to velocity for {node_id}: {e}"))?;
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6040, 0), 0x06, timeout).await
		.map_err(|e| log::error!("Failed to enable voltage to motor of node {node_id}: {e}"))?;
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6040, 0), 0x06, timeout).await
		.map_err(|e| log::error!("Failed to switch on motor of node {node_id}: {e}"))?;
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6040, 0), 0x0F, timeout).await
		.map_err(|e| log::error!("Failed to enable operation of node {node_id}: {e}"))?;

	log::info!("Setting speed to 200");
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6042, 0), 200, timeout).await
		.map_err(|e| log::error!("Failed to set target speed of node {node_id}: {e}"))?;

	tokio::time::sleep(Duration::from_secs(1)).await;

	log::info!("Setting speed to 0");
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6042, 0), 0, timeout).await
		.map_err(|e| log::error!("Failed to set target speed of node {node_id}: {e}"))?;
	Ok(())
}

fn parse_timeout(input: &str) -> Result<Duration, &'static str> {
	let seconds: f64 = input.parse()
		.map_err(|_| "invalid duration: expected timeout in seconds")?;
	Ok(Duration::from_secs_f64(seconds))
}

fn parse_number<T: TryFrom<i128>>(input: &str) -> Result<T, String>
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
