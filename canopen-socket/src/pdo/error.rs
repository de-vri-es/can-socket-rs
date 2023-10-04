use crate::sdo::SdoError;

/// An error that can occur when getting or setting a PDO mapping.
#[derive(Debug)]
#[derive(thiserror::Error)]
#[error("{0}")]
pub enum PdoConfigError {
	/// The PDO number is not valid.
	InvalidPdoNumber(#[from] InvalidPdoNumber),

	/// An error occured when trying to access the configuration.
	SdoError(#[from] SdoError),
}

/// The PDO number is not valid.
#[derive(Debug, Clone)]
#[derive(thiserror::Error)]
#[error("invalid PDO number: value must be between 0 and 511 (inclusive), but got {value}")]
pub struct InvalidPdoNumber {
	pub(super) value: u16,
}

/// The value for the `nth sync` PDO mode is invalid.
#[derive(Debug, Clone)]
#[derive(thiserror::Error)]
#[error("invalid value for PDO mode `nth sync`: value must be between 1 and 240 (inclusive), but got {value}")]
pub struct InvalidSyncInterval {
	pub(super) value: u8,
}

impl From<crate::sdo::UploadError<std::convert::Infallible>> for PdoConfigError {
	fn from(value: crate::sdo::UploadError<std::convert::Infallible>) -> Self {
		match value {
			crate::sdo::UploadError::UploadFailed(e) => e.into(),
			crate::sdo::UploadError::ParseFailed(_) => unreachable!(),
		}
	}
}
