use can_socket::{CanFrame, CanId};
use can_socket::tokio::CanSocket;
use canopen_socket::{CanOpenSocket, ObjectIndex};
use canopen_socket::pdo::{
	PdoMapping,
	RpdoConfiguration,
	RpdoTransmissionType,
	TpdoConfiguration,
	TpdoTransmissionType,
};
use canopen_socket::sdo::SdoAddress;
use std::time::{Duration, Instant};

#[derive(clap::Parser)]
struct Options {
	/// The CAN interface to use.
	interface: String,

	/// The node read from.
	#[clap(value_parser(parse_number::<u8>))]
	node_id: u8,

	/// The maximum current in mA.
	#[clap(long)]
	max_current: Option<u32>,

	/// The velocity to set.
	#[clap(long)]
	velocity: u16,

	/// The time to ramp up and down the speed.
	#[clap(long)]
	#[clap(value_parser(parse_duration))]
	#[clap(default_value = "1")]
	ramp_time: Duration,

	/// The time to hold the target speed after the ramp up and before the ramp down.
	#[clap(long)]
	#[clap(value_parser(parse_duration))]
	#[clap(default_value = "3")]
	hold_time: Duration,

	/// The step time for one SYNC cycle.
	#[clap(long)]
	#[clap(value_parser(parse_duration))]
	#[clap(default_value = "0.01")]
	step_time: Duration,

	/// Timeout in seconds for receiving the reply.
	#[clap(long, short)]
	#[clap(value_parser(parse_duration))]
	#[clap(default_value = "1")]
	timeout: Duration,
}

#[tokio::main]
async fn main() {
	// Initialize the logging system.
	env_logger::builder()
		.filter_module(module_path!(), log::LevelFilter::Info)
		.parse_default_env()
		.init();

	// Run `do_main()` and report a possible error by exitting with status 1.
	if let Err(()) = do_main(clap::Parser::parse()).await {
		std::process::exit(1);
	}
}

async fn do_main(options: Options) -> Result<(), ()> {
	// Open the CAN socket and wrap it in a CanOpenSocket.
	let socket = CanSocket::bind(&options.interface)
		.map_err(|e| log::error!("Failed to create CAN socket for interface {}: {e}", options.interface))?;
	let mut socket = CanOpenSocket::new(socket);

	let node_id = options.node_id;
	let timeout = options.timeout;
	let sdo = SdoAddress::standard();

	// Reset the motor controller.
	socket.send_nmt_command(node_id, canopen_socket::nmt::NmtCommand::ResetCommunication, timeout).await
		.map_err(|e| log::error!("Failed to reset communication of node {node_id}: {e}"))?;

	// Configure TPDO 0.
	let tpdo0_config = TpdoConfiguration {
		communication: canopen_socket::pdo::TpdoCommunicationParameters {
			enabled: true,
			rtr_allowed: true,
			cob_id: CanId::new(0x181).unwrap(),
			mode: TpdoTransmissionType::sync(1).unwrap(), // Send the PDO on every SYNC message.
			inhibit_time_100us: 0,
			event_timer_ms: 0,
			start_sync: 0,
		},
		mapping: vec![
			// Include the Statusword object from 0x6041.
			PdoMapping {
				object: ObjectIndex::new(0x6041, 0),
				bit_length: 16,
			},
			// Include the VI Velocity Demand object from 0x6043.
			PdoMapping {
				object: ObjectIndex::new(0x6043, 0),
				bit_length: 16,
			},
			// Include the actual motor current from 0x2039 subindex 5.
			PdoMapping {
				object: ObjectIndex::new(0x2039, 5),
				bit_length: 32,
			},
		],
	};
	socket.configure_tpdo(node_id, sdo, 0, &tpdo0_config, timeout).await
		.map_err(|e| log::error!("Failed to configure TPDO 0 of node {node_id}: {e}"))?;

	// Configure RPDO 0.
	let rpdo0_config = RpdoConfiguration {
		communication: canopen_socket::pdo::RpdoCommunicationParameters {
			enabled: true,
			cob_id: CanId::new(0x201).unwrap(),
			mode: RpdoTransmissionType::sync(), // Actuate the PDO value at every SYNC.
			inhibit_time_100us: 0,
			deadline_timer_ms: 0,
		},
		mapping: vec![
			// Include the VI Target Velocity object from 0x6004.
			PdoMapping {
				object: ObjectIndex::new(0x6042, 0),
				bit_length: 16,
			},
		],
	};
	socket.configure_rpdo(node_id, sdo, 0, &rpdo0_config, timeout).await
		.map_err(|e| log::error!("Failed to configure RPDO 0 of node {node_id}: {e}"))?;

	// Start the motor controller.
	socket.send_nmt_command(node_id, canopen_socket::nmt::NmtCommand::Start, timeout).await
		.map_err(|e| log::error!("Failed to start node {node_id}: {e}"))?;

	// Set maximum motor current.
	if let Some(max_current) = options.max_current {
		socket.sdo_download::<u32>(node_id, sdo, ObjectIndex::new(0x2031, 0), max_current, timeout).await
			.map_err(|e| log::error!("Failed to set maxmimum current of node {node_id}: {e}"))?;
	}

	// Check the Statusword object and log a warning.
	let status: u16 = socket.sdo_upload(node_id, sdo, ObjectIndex::new(0x6041, 0), timeout).await
		.map_err(|e| log::error!("Failed to read device status of node {node_id}: {e}"))?;
	if status & 0x08 != 0 {
		log::warn!("Device is in fault mode.");
	}

	// Clear the target velocity, configure velocity mode and enable the motor.
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6042, 0), 0, timeout).await
		.map_err(|e| log::error!("Failed to set velocity target of node {node_id}: {e}"))?;
	socket.sdo_download::<u8>(node_id, sdo, ObjectIndex::new(0x6060, 0), 2, timeout).await
		.map_err(|e| log::error!("Failed to set operation mode to velocity for {node_id}: {e}"))?;
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6040, 0), 0x06, timeout).await
		.map_err(|e| log::error!("Failed to enable voltage to motor of node {node_id}: {e}"))?;
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6040, 0), 0x07, timeout).await
		.map_err(|e| log::error!("Failed to switch on motor of node {node_id}: {e}"))?;
	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6040, 0), 0x0F, timeout).await
		.map_err(|e| log::error!("Failed to enable operation of node {node_id}: {e}"))?;

	let result = do_loop(&mut socket, &options).await;

	socket.sdo_download::<u16>(node_id, sdo, ObjectIndex::new(0x6040, 0), 0x06, timeout).await
		.map_err(|e| log::error!("Failed to disable motor of node {node_id}: {e}"))?;
	result?;

	Ok(())
}

async fn do_loop(socket: &mut CanOpenSocket, options: &Options) -> Result<(), ()> {
	let mut deadline = Instant::now();
	let velocity = options.velocity;
	let ramp_time: f64 = options.ramp_time.as_secs_f64();
	let hold_time: f64 = options.hold_time.as_secs_f64();
	let step_time: f64 = options.step_time.as_secs_f64();

	let ramp_steps = (ramp_time / step_time).round() as u16;
	let hold_steps = (hold_time / step_time).round() as u32;
	let velocity_step = velocity / ramp_steps;

	for i in 0..ramp_steps {
		deadline += options.step_time;
		do_step(socket, i * velocity_step, deadline).await?;
		tokio::time::sleep_until(deadline.into()).await;
	}
	for _ in 0..hold_steps {
		deadline += options.step_time;
		do_step(socket, velocity, deadline).await?;
		tokio::time::sleep_until(deadline.into()).await;
	}
	for i in (0..ramp_steps).rev() {
		deadline += options.step_time;
		do_step(socket, i * velocity_step, deadline).await?;
		tokio::time::sleep_until(deadline.into()).await;
	}

	Ok(())
}

/// Do one control step:
///   * Send the velocity target PDO.
///   * Send a SYNC command.
///   * Receive the feedback PDO.
async fn do_step(socket: &mut CanOpenSocket, velocity_target: u16, deadline: Instant) -> Result<(), ()> {
	socket.send_frame(&CanFrame::new(CanId::new(0x201).unwrap(), &velocity_target.to_le_bytes(), None).unwrap())
		.await
		.map_err(|e| log::error!("Failed to send PDO 0x201: {e}"))?;
	socket.send_sync(None)
		.await
		.map_err(|e| log::error!("Failed to send SYNC: {e}"))?;
	let reply = socket.recv_frame_deadline(deadline)
		.await
		.ok_or_else(|| log::error!("Timeout reached before receiving PDO"))?
		.map_err(|e| log::error!("Failed to receive CAN frame: {e}"))?;
	let status = parse_pdo(&reply)?;
	log::info!("status: 0x{:02X}, velocity demand: {:5} rpm, motor current: {:5} mA", status.status, status.velocity_demand, status.current_actual as f64);
	Ok(())
}

/// Status information from RPDO 0.
struct Status {
	status: u16,
	velocity_demand: u16,
	current_actual: u32,
}

/// Parse a CAN frame as the expected PDO.
fn parse_pdo(input: &CanFrame) -> Result<Status, ()> {
	let id = input.id();
	let data = input.data();

	// Check the CAN ID.
	if id.as_u32() != 0x181 {
		log::error!("Received unexpected CAN frame with ID: {id:?}");
		return Err(());
	}

	// Check the data length.
	if data.len() != 8 {
		log::error!("PDO message has wrong length, expected 8 data bytes, got {}", data.len());
		return Err(());
	}

	// Extract the fields.
	let status = u16::from_le_bytes([data[0], data[1]]);
	let velocity_demand = u16::from_le_bytes([data[2], data[3]]);
	let current_actual = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
	Ok(Status { status, velocity_demand, current_actual })
}

fn parse_duration(input: &str) -> Result<Duration, &'static str> {
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
