use can_socket::tokio::CanSocket;
use canopen_tokio::CanOpenSocket;
use canopen_tokio::ObjectIndex;
use canopen_tokio::sdo::SdoAddress;
use std::time::Duration;

#[derive(clap::Parser)]
struct Options {
	/// The CAN interface to use.
	interface: String,

	/// The node read from.
	#[clap(value_parser(parse_number::<u8>))]
	node_id: u8,

	/// The object index to read from.
	#[clap(value_parser(parse_number::<u16>))]
	index: u16,

	/// The object subindex to read from.
	#[clap(value_parser(parse_number::<u8>))]
	subindex: u8,

	#[clap(long, short)]
	#[clap(value_enum)]
	#[clap(default_value = "hexadecimal")]
	format: Format,

	/// Timeout in seconds for receiving the reply.
	#[clap(long, short)]
	#[clap(value_parser(parse_timeout))]
	#[clap(default_value = "1")]
	timeout: Duration,
}

#[derive(clap::ValueEnum)]
#[derive(Debug, Clone)]
enum Format {
	Raw,
	#[clap(alias = "oct")]
	Octal,
	#[clap(alias = "hex")]
	Hexadecimal,
	#[clap(alias = "dec")]
	Decimal,
	Utf8,
	Utf16,
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
	let data: Vec<u8> = socket.sdo_upload(options.node_id, SdoAddress::standard(), object, options.timeout).await
		.map_err(|e| log::error!("{e}"))?;

	display_data(options.format, &data)?;
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

fn display_data(format: Format, data: &[u8]) -> Result<(), ()> {
	use std::io::Write;
	match format {
		Format::Raw => {
			std::io::stdout().write_all(data)
				.map_err(|e| log::error!("Failed to write to stdout: {e}"))?;
		},
		Format::Octal => {
			display_bytes(ByteStyle::Octal, data)
				.map_err(|e| log::error!("Failed to write to stdout: {e}"))?;
		},
		Format::Decimal => {
			display_bytes(ByteStyle::Decimal, data)
				.map_err(|e| log::error!("Failed to write to stdout: {e}"))?;
		},
		Format::Hexadecimal => {
			display_bytes(ByteStyle::Hexadecimal, data)
				.map_err(|e| log::error!("Failed to write to stdout: {e}"))?;
		},
		Format::Utf8 => {
			let data = std::str::from_utf8(data)
				.map_err(|e| log::error!("invalid UTF-8 in string data: {e}"))?;
			println!("{data}");
		},
		Format::Utf16 => {
			let data = std::str::from_utf8(data)
				.map_err(|e| log::error!("invalid UTF-8 in string data: {e}"))?;
			println!("{data}");
		},
	}
	Ok(())
}

enum ByteStyle {
	Octal,
	Decimal,
	Hexadecimal,
}

fn display_bytes(style: ByteStyle, data: &[u8]) -> std::io::Result<()> {
	use std::io::Write;
	let stdout = std::io::stdout();
	let mut stdout = stdout.lock();
	let mut bytes = data.iter();
	loop {
		for i in 0..20 {
			let Some(byte) = bytes.next() else {
				break;
			};
			if i != 0 {
				stdout.write_all(b" ")?;
			}
			match style {
				ByteStyle::Octal => write!(stdout, "{byte:03o}")?,
				ByteStyle::Decimal => write!(stdout, "{byte:3}")?,
				ByteStyle::Hexadecimal => write!(stdout, "{byte:02X}")?,
			}
		}
		stdout.write_all(b"\n")?;
		if bytes.len() == 0 {
			break;
		}
	}
	Ok(())
}
