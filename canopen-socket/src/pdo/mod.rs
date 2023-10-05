//! Process Data Object (PDO) types and utilities.

mod error;
pub use error::*;

mod read_config;
pub(crate) use read_config::*;

mod types;
pub use types::*;

mod write_config;
pub(crate) use write_config::*;

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
