use std::time::Duration;

use can_socket::tokio::CanSocket;
use canopen_socket::{CanOpenSocket, nmt::NmtCommand};

#[derive(clap::Parser)]
struct Options {
	/// The interface to read from.
	interface: String,

	/// The node to command.
	node_id: u8,

	/// The command to send.
	command: Command,

	/// Timeout in seconds for receiving the reply.
	#[clap(long, short)]
	#[clap(value_parser(parse_timeout))]
	#[clap(default_value = "1")]
	timeout: Duration,
}

#[derive(clap::ValueEnum)]
#[derive(Debug, Clone)]
enum Command {
	Start,
	Stop,
	GoToPreOperational,
	Reset,
	ResetCommunication,
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

	let command = match options.command {
		Command::Start => NmtCommand::Start,
		Command::Stop => NmtCommand::Stop,
		Command::GoToPreOperational => NmtCommand::GoToPreOperational,
		Command::Reset => NmtCommand::Reset,
		Command::ResetCommunication => NmtCommand::ResetCommunication,
	};

	socket.send_nmt_command(options.node_id, command, options.timeout).await
		.map_err(|e| eprintln!("{e}"))?;

	eprintln!("OK");
	Ok(())
}

fn parse_timeout(input: &str) -> Result<Duration, &'static str> {
	let seconds: f64 = input.parse()
		.map_err(|_| "invalid duration: expected timeout in seconds")?;
	Ok(Duration::from_secs_f64(seconds))
}
