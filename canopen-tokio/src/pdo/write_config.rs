use std::time::Duration;

use can_socket::CanId;

use crate::sdo::SdoAddress;
use crate::{ObjectIndex, CanOpenSocket};

use super::{
	PdoConfigError,
	PdoMapping,
	RpdoCommunicationParameters,
	RpdoConfiguration,
	TpdoCommunicationParameters,
	TpdoConfiguration,
};

/// Enable or disable an RPDO.
pub(crate) async fn enable_rpdo(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	enabled: bool,
	timeout: Duration,
) -> Result<(), PdoConfigError> {
	let index = super::rpdo_communication_params_object(pdo)?;
	let object = ObjectIndex::new(index, 1);
	let cob_id: u32 = bus.sdo_upload(node_id, sdo, object, timeout).await?;
	let cob_id = if enabled {
		cob_id & !0x8000_0000 // Disable bit 31 to enable the PDO.
	} else {
		cob_id | 0x8000_0000 // Enable bit 31 to disable the PDO.
	};
	bus.sdo_download(node_id, sdo, object, cob_id, timeout).await?;
	Ok(())
}

/// Enable or disable an RPDO.
pub(crate) async fn enable_tpdo(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	enabled: bool,
	timeout: Duration,
) -> Result<(), PdoConfigError> {
	let index = super::tpdo_communication_params_object(pdo)?;
	let object = ObjectIndex::new(index, 1);
	let cob_id: u32 = bus.sdo_upload(node_id, sdo, object, timeout).await?;
	let cob_id = if enabled {
		cob_id & !0x8000_0000 // Disable bit 31 to enable the PDO.
	} else {
		cob_id | 0x8000_0000 // Enable bit 31 to disable the PDO.
	};
	bus.sdo_download(node_id, sdo, object, cob_id, timeout).await?;
	Ok(())
}

/// Set the full configuration of an RPDO.
pub(crate) async fn configure_rpdo(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	config: &RpdoConfiguration,
	timeout: Duration,
) -> Result<(), PdoConfigError> {
	let mapping_index = super::rpdo_mapping_object(pdo)?;
	enable_rpdo(bus, node_id, sdo, pdo, false, timeout).await?;

	write_rpdo_communication_parameters(bus, node_id, sdo, pdo, &config.communication, timeout).await?;
	configure_pdo_mapping(bus, node_id, sdo, mapping_index, &config.mapping, timeout).await?;

	if config.communication.enabled {
		enable_rpdo(bus, node_id, sdo, pdo, true, timeout).await?
	}

	Ok(())
}

/// Read the configuration of a TPDO.
pub(crate) async fn configure_tpdo(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	config: &TpdoConfiguration,
	timeout: Duration,
) -> Result<(), PdoConfigError> {
	let mapping_index = super::tpdo_mapping_object(pdo)?;
	enable_tpdo(bus, node_id, sdo, pdo, false, timeout).await?;

	write_tpdo_communication_parameters(bus, node_id, sdo, pdo, &config.communication, timeout).await?;
	configure_pdo_mapping(bus, node_id, sdo, mapping_index, &config.mapping, timeout).await?;

	if config.communication.enabled {
		enable_tpdo(bus, node_id, sdo, pdo, true, timeout).await?
	}

	Ok(())
}

/// Read the communication parameters of an RPDO.
pub(crate) async fn write_rpdo_communication_parameters(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	params: &RpdoCommunicationParameters,
	timeout: Duration,
) -> Result<(), PdoConfigError> {
	let config_index = super::rpdo_communication_params_object(pdo)?;

	// Valid subindices should be a `u8` according to the CANopen documentation,
	// but at least one device (the CANit-10) sends back 4 bytes instead.
	// Requesting a `u32` seems to work on other devices too.
	let valid_subindices: u32 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 0), timeout).await?; 
	if valid_subindices < 3 && params.inhibit_time_100us > 0 {
		return Err(PdoConfigError::InhibitTimeNotSupported);
	}
	if valid_subindices < 5 && params.deadline_timer_ms > 0 {
		return Err(PdoConfigError::DeadlineTimerNotSupported);
	}

	let cob_id = match params.cob_id {
		CanId::Standard(x) => u32::from(x.as_u16()) | (1 << 31), // Keep bit 31 enabled to so that the PDO is still disabled.
		CanId::Extended(x) => x.as_u32() | (1 << 29) | (1 << 31), // For extended CAN IDs also enable bit 29.
	};

	bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 1), cob_id, timeout).await?;
	bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 2), params.mode.to_u8(), timeout).await?;
	if valid_subindices >= 3 {
		bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 3), params.inhibit_time_100us, timeout).await?
	};
	if valid_subindices >= 5 {
		bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 5), params.deadline_timer_ms, timeout).await?
	};

	Ok(())
}

/// Read the communication parameters of a TPDO.
pub(crate) async fn write_tpdo_communication_parameters(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	params: &TpdoCommunicationParameters,
	timeout: Duration,
) -> Result<(), PdoConfigError> {
	let config_index = super::tpdo_communication_params_object(pdo)?;

	// Valid subindices should be a `u8` according to the CANopen documentation,
	// but at least one device (the CANit-10) sends back 4 bytes instead.
	// Requesting a `u32` seems to work on other devices too.
	let valid_subindices: u32 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 0), timeout).await?;
	if valid_subindices < 3 && params.inhibit_time_100us > 0 {
		return Err(PdoConfigError::InhibitTimeNotSupported);
	}
	if valid_subindices < 5 && params.event_timer_ms > 0 {
		return Err(PdoConfigError::EventTimerNotSupported);
	}
	if valid_subindices < 6 && params.start_sync > 0 {
		return Err(PdoConfigError::StartSyncNotSupported);
	}

	let cob_id = match params.cob_id {
		CanId::Standard(x) => u32::from(x.as_u16()) | (1 << 31), // Keep bit 31 enabled to so that the PDO is still disabled.
		CanId::Extended(x) => x.as_u32() | (1 << 29) | (1 << 31), // For extended CAN IDs also enable bit 29.
	};

	let cob_id = match params.rtr_allowed {
		true => cob_id & !(1 << 30), // Clearing bit 30 allows remote frame requests.
		false => cob_id | (1 << 30), // Setting bit 30 high disallows remote frame requests.
	};

	bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 1), cob_id, timeout).await?;
	bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 2), params.mode.to_u8(), timeout).await?;
	if valid_subindices >= 3 {
		bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 3), params.inhibit_time_100us, timeout).await?
	};
	if valid_subindices >= 5 {
		bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 5), params.event_timer_ms, timeout).await?
	};
	if valid_subindices >= 6 {
		bus.sdo_download(node_id, sdo, ObjectIndex::new(config_index, 6), params.start_sync, timeout).await?
	};

	Ok(())
}

/// Configure the mapping of a PDO object (RPDO or TPDO).
pub(crate) async fn configure_pdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	object_index: u16,
	mappings: &[PdoMapping],
	timeout: Duration,
) -> Result<(), PdoConfigError> {
	// Set field count to 0.
	bus.sdo_download(node_id, sdo, ObjectIndex::new(object_index, 0), 0u8, timeout).await?;

	for (i, mapping) in mappings.iter().enumerate() {
		bus.sdo_download(node_id, sdo, ObjectIndex::new(object_index, i as u8 + 1), mapping.to_u32(), timeout).await?;
	}

	// Set field count correctly.
	bus.sdo_download(node_id, sdo, ObjectIndex::new(object_index, 0), mappings.len() as u8, timeout).await?;
	Ok(())
}
