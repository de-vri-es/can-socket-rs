use std::time::Duration;

use can_socket::CanId;

use crate::sdo::SdoAddress;
use crate::{ObjectIndex, CanOpenSocket};

use super::{
	PdoConfigError, PdoMapping, RpdoCommunicationParameters, RpdoConfiguration, RpdoKind, RpdoTransmissionType, TpdoCommunicationParameters, TpdoConfiguration, TpdoKind, TpdoTransmissionType
};

/// Read the configuration of an RPDO.
pub(crate) async fn read_rpdo_configuration(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	kind: RpdoKind,
	timeout: Duration,
) -> Result<RpdoConfiguration, PdoConfigError> {
	let mapping_index = super::rpdo_mapping_object(kind)?;
	let communication = read_rpdo_communication_parameters(bus, node_id, sdo, kind, timeout).await?;
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
	kind: TpdoKind,
	timeout: Duration,
) -> Result<TpdoConfiguration, PdoConfigError> {
	let mapping_index = super::tpdo_mapping_object(kind)?;
	let communication = read_tpdo_communication_parameters(bus, node_id, sdo, kind, timeout).await?;
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
	kind: RpdoKind,
	timeout: Duration,
) -> Result<RpdoCommunicationParameters, PdoConfigError> {
	let config_index = super::rpdo_communication_params_object(kind)?;

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
	kind: TpdoKind,
	timeout: Duration,
) -> Result<TpdoCommunicationParameters, PdoConfigError> {
	let config_index = super::tpdo_communication_params_object(kind)?;

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

	let enabled = cob_id & (1 << 31) == 0; // bit 31 value 0 indicates PDO is enabled.
	let rtr_allowed = cob_id & (1 << 30) == 0; // bit 30 value 0 indicates RTR is allowed.
	let cob_id = CanId::new(cob_id & 0x1FFF_FFFF).unwrap();
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
		let mapping: u32 = bus.sdo_upload(node_id, sdo, ObjectIndex::new(object_index, i), timeout).await?;
		fields.push(PdoMapping::from_u32(mapping))
	}

	Ok(fields)
}
