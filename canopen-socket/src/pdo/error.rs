use crate::sdo::SdoError;

/// An error that can occur when getting or setting a PDO mapping.
#[derive(Debug)]
pub enum PdoMappingError {
	/// The PDO number is not valid.
	InvalidPdoNumber(InvalidPdoNumber),

	/// An error occured when trying to access the configuration.
	SdoError(SdoError),
}

/// The PDO number is not valid.
#[derive(Debug, Clone)]
#[derive(thiserror::Error)]
#[error("invalid PDO number: value must be between 0 and 511 (inclusive), but got {value}")]
pub struct InvalidPdoNumber {
	pub(super) value: u8,
}

/// The value for the `nth sync` PDO mode is invalid.
#[derive(Debug, Clone)]
#[derive(thiserror::Error)]
#[error("invalid value for PDO mode `nth sync`: value must be between 2 and 240 (inclusive), but got {value}")]
pub struct InvalidNthSyncCounter {
	pub(super) value: u8,
}
