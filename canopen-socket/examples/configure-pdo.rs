use can_socket::tokio::CanSocket;
use canopen_socket::pdo::PdoMapping;
use canopen_socket::{CanOpenSocket, ObjectIndex};
use canopen_socket::sdo::SdoAddress;
use std::time::Duration;

#[derive(clap::Parser)]
struct Options {
	/// The CAN interface to use.
	interface: String,

	/// The node read from.
	#[clap(value_parser(parse_number::<u8>))]
	node_id: u8,

	/// The type of PDO.
	#[clap(value_enum)]
	#[clap(name = "type")]
	kind: PdoType,

	/// The PDO number to read the mapping of.
	#[clap(value_parser(parse_number::<u16>))]
	pdo: u16,

	/// Enable th PDO.
	#[clap(long)]
	#[clap(group = "on-off")]
	enable: bool,

	/// Disable the PDO.
	#[clap(long)]
	#[clap(group = "on-off")]
	disable: bool,

	#[clap(long)]
	#[clap(group = "mappings")]
	clear_mappings: bool,

	#[clap(long)]
	#[clap(group = "mappings")]
	#[clap(value_parser = parse_pdo_mapping)]
	#[clap(value_name = "INDEX,SUBINDEX,BITS")]
	mapping: Vec<PdoMapping>,

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

	match options.kind {
		PdoType::Rpdo => {
			let mut config = socket.read_rpdo_configuration(options.node_id, SdoAddress::standard(), options.pdo, options.timeout).await
				.map_err(|e| log::error!("Failed to read configuration of RPDO {} of node {}: {e}", options.pdo, options.node_id))?;
			if options.clear_mappings {
				config.mapping.clear();
			} else if !options.mapping.is_empty() {
				config.mapping = options.mapping;
			}
			if options.enable {
				config.communication.enabled = true;
			} else if options.disable {
				config.communication.enabled = false;
			}
			log::info!("Setting RPDO configuration: {config:#?}");
			socket.configure_rpdo(options.node_id, SdoAddress::standard(), options.pdo, &config, options.timeout).await
				.map_err(|e| log::error!("Failed to configure RPDO {} of node {}: {e}", options.pdo, options.node_id))?;
		},
		PdoType::Tpdo => {
			let mut config = socket.read_tpdo_configuration(options.node_id, SdoAddress::standard(), options.pdo, options.timeout).await
				.map_err(|e| log::error!("Failed to read configuration of RPDO {} of node {}: {e}", options.pdo, options.node_id))?;
			if options.clear_mappings {
				config.mapping.clear();
			} else if !options.mapping.is_empty() {
				config.mapping = options.mapping;
			}
			if options.enable {
				config.communication.enabled = true;
			} else if options.disable {
				config.communication.enabled = false;
			}
			log::info!("Setting TPDO configuration: {config:#?}");
			socket.configure_tpdo(options.node_id, SdoAddress::standard(), options.pdo, &config, options.timeout).await
				.map_err(|e| log::error!("Failed to configure RPDO {} of node {}: {e}", options.pdo, options.node_id))?;
		},
	}

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

fn parse_pdo_mapping(input: &str) -> Result<PdoMapping, String> {
	let (index, tail) = input.split_once(',')
		.ok_or_else(|| format!("invalid mapping: {input:?}: expected INDEX,SUBINDEX,BITS"))?;
	let (subindex, bit_length) = tail.split_once(',')
		.ok_or_else(|| format!("invalid mapping: {input:?}: expected INDEX,SUBINDEX,BITS"))?;
	let index = parse_number(index)?;
	let subindex = parse_number(subindex)?;
	let bit_length = parse_number(bit_length)?;
	Ok(PdoMapping {
		object: ObjectIndex::new(index, subindex),
		bit_length,
	})
}
