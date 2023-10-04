use std::time::Duration;

use crate::sdo::SdoAddress;
use crate::{ObjectIndex, CanOpenSocket};

use super::{PdoMappingError, InvalidPdoNumber, RpdoMapping, TpdoMapping};

pub(crate) async fn read_rpdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo: SdoAddress,
	pdo: u16,
	timeout: Duration,
) -> Result<RpdoMapping, PdoMappingError> {
	let config_index = rpdo_config_object(pdo)?;
	let mapping_index = rpdo_mapping_object(pdo)?;

	let cob_id: u32 = bus.sdo_upload::<u32>(node_id, sdo, ObjectIndex::new(config_index, 0), timeout).await?;
	todo!()
}

pub(crate) fn read_tpdo_mapping(
	bus: &mut CanOpenSocket,
	sdo_address: SdoAddress,
	node_id: u8,
	pdo: u16,
	timeout: Duration,
) -> Result<TpdoMapping, PdoMappingError> {
	todo!()
}

pub(crate) fn set_rpdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo_address: SdoAddress,
	pdo: u16,
	mapping: RpdoMapping,
) -> Result<(), PdoMappingError> {
	todo!()
}

pub(crate) fn set_tpdo_mapping(
	bus: &mut CanOpenSocket,
	node_id: u8,
	sdo_address: SdoAddress,
	pdo: u16,
	mapping: TpdoMapping,
) -> Result<(), PdoMappingError> {
	todo!()
}

fn rpdo_config_object(pdo: u16) -> Result<u16, InvalidPdoNumber> {
	if pdo < 512 {
		Ok(0x1400 + pdo)
	} else {
		Err(InvalidPdoNumber { value: pdo })
	}
}

fn rpdo_mapping_object(pdo: u16) -> Result<u16, InvalidPdoNumber> {
	if pdo < 512 {
		Ok(0x1600 + pdo)
	} else {
		Err(InvalidPdoNumber { value: pdo })
	}
}

fn tpdo_config_object(pdo: u16) -> Result<u16, InvalidPdoNumber> {
	if pdo < 512 {
		Ok(0x1800 + pdo)
	} else {
		Err(InvalidPdoNumber { value: pdo })
	}
}

fn tpdo_mapping_object(pdo: u16) -> Result<u16, InvalidPdoNumber> {
	if pdo < 512 {
		Ok(0x1A00 + pdo)
	} else {
		Err(InvalidPdoNumber { value: pdo })
	}
}
