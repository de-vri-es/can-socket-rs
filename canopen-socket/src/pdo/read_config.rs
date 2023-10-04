use std::time::Duration;

use can_socket::CanId;

use crate::sdo::SdoAddress;
use crate::{ObjectIndex, CanOpenSocket};

use super::{
	InvalidPdoNumber,
	PdoConfigError,
	PdoMapping,
	RpdoCommunicationParameters,
	RpdoConfiguration,
	RpdoTransmissionType,
	TpdoCommunicationParameters,
	TpdoConfiguration,
	TpdoTransmissionType,
};

/// Read the configuration of an RPDO.
pub(crate) async fn read_rpdo_configuration(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	timeout: Duration,
) -> Result<RpdoConfiguration, PdoConfigError> {
	let mapping_index = rpdo_mapping_object(pdo)?;
	let communication = read_rpdo_communication_parameters(bus, node_id, sdo, pdo, timeout).await?;
	let mapping = read_pdo_mapping(bus, node_id, sdo, mapping_index, timeout).await?;

	Ok(RpdoConfiguration {
		communication,
		mapping,
	})
}

/// Read the configuration of a TPDO.
pub(crate) async fn read_tpdo_configuration(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	timeout: Duration,
) -> Result<TpdoConfiguration, PdoConfigError> {
	let mapping_index = tpdo_mapping_object(pdo)?;
	let communication = read_tpdo_communication_parameters(bus, node_id, sdo, pdo, timeout).await?;
	let mapping = read_pdo_mapping(bus, node_id, sdo, mapping_index, timeout).await?;

	Ok(TpdoConfiguration {
		communication,
		mapping,
	})
}

/// Read the communication parameters of an RPDO.
pub(crate) async fn read_rpdo_communication_parameters(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	timeout: Duration,
) -> Result<RpdoCommunicationParameters, PdoConfigError> {
	let config_index = rpdo_communication_params_object(pdo)?;

	let valid_subindices: u8 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 0), timeout).await?;
	let cob_id: u32 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 1), timeout).await?;
	let mode: u8 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 2), timeout).await?;
	let inhibit_time_100us: u16 = if valid_subindices >= 3 {
		bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 3), timeout).await?
	} else {
		0
	};
	let deadline_timer_ms: u16 = if valid_subindices >= 5 {
		bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 5), timeout).await?
	} else {
		0
	};

	let enabled = cob_id & 0x8000_0000 == 0; // bit value 0 indicates PDO is enabled.
	let cob_id = CanId::new(cob_id & 0x1000_0000).unwrap();
	let mode = RpdoTransmissionType::from_u8(mode);

	Ok(RpdoCommunicationParameters {
		enabled,
		cob_id,
		mode,
		inhibit_time_100us,
		deadline_timer_ms,
	})
}

/// Read the communication parameters of a TPDO.
pub(crate) async fn read_tpdo_communication_parameters(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	timeout: Duration,
) -> Result<TpdoCommunicationParameters, PdoConfigError> {
	let config_index = tpdo_communication_params_object(pdo)?;

	let valid_subindices: u8 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 0), timeout).await?;
	let cob_id: u32 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 1), timeout).await?;
	let mode: u8 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 2), timeout).await?;
	let inhibit_time_100us: u16 = if valid_subindices >= 3 {
		bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 3), timeout).await?
	} else {
		0
	};
	let event_timer_ms: u16 = if valid_subindices >= 5 {
		bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 5), timeout).await?
	} else {
		0
	};
	let start_sync: u8 = if valid_subindices >= 6 {
		bus.sdo_upload(node_id, sdo, ObjectIndex::new(config_index, 6), timeout).await?
	} else {
		0
	};

	let enabled = cob_id & 0x8000_0000 == 0; // bit value 0 indicates PDO is enabled.
	let rtr_allowed = cob_id & 0x4000_0000 == 0; // bit value 0 indicates RTR is allowed.
	let cob_id = CanId::new(cob_id & 0x1000_0000).unwrap();
	let mode = TpdoTransmissionType::from_u8(mode);

	Ok(TpdoCommunicationParameters {
		enabled,
		rtr_allowed,
		cob_id,
		mode,
		inhibit_time_100us,
		event_timer_ms,
		start_sync,
	})
}

/// Read the mapping of a PDO object (RPDO or TPDO).
pub(crate) async fn read_pdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	object_index: u16,
	timeout: Duration,
) -> Result<Vec<PdoMapping>, PdoConfigError> {
	let count: u8 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(object_index, 0), timeout).await?;
	let mut fields = Vec::with_capacity(count.into());

	for i in 1..=count {
		let field: u32 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(object_index, i), timeout).await?;
		let index = (field >> 16) as u16;
		let sub_index = (field >> 8 & 0xFF) as u8;
		let bit_length = (field & 0xFF) as u8;
		fields.push(PdoMapping {
			object: ObjectIndex::new(index, sub_index),
			bit_length,
		})
	}

	Ok(fields)
}

/// Get the object index of the communication parameters of an RPDO.
fn rpdo_communication_params_object(pdo: u16) -> Result<u16, InvalidPdoNumber> {
	if pdo < 512 {
		Ok(0x1400 + pdo)
	} else {
		Err(InvalidPdoNumber { value: pdo })
	}
}

/// Get the object index of the mapping parameters of an RPDO.
fn rpdo_mapping_object(pdo: u16) -> Result<u16, InvalidPdoNumber> {
	if pdo < 512 {
		Ok(0x1600 + pdo)
	} else {
		Err(InvalidPdoNumber { value: pdo })
	}
}

/// Get the object index of the communication parameters of a TPDO.
fn tpdo_communication_params_object(pdo: u16) -> Result<u16, InvalidPdoNumber> {
	if pdo < 512 {
		Ok(0x1800 + pdo)
	} else {
		Err(InvalidPdoNumber { value: pdo })
	}
}

/// Get the object index of the mapping parameters of a TPDO.
fn tpdo_mapping_object(pdo: u16) -> Result<u16, InvalidPdoNumber> {
	if pdo < 512 {
		Ok(0x1A00 + pdo)
	} else {
		Err(InvalidPdoNumber { value: pdo })
	}
}
