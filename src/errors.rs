//! Error types used in the gerber-types library.

use std::io::Error as IoError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GerberError {
    #[error("Conversion between two types failed: {0}")]
    ConversionError(String),

    #[error("Bad coordinate format: {0}")]
    CoordinateFormatError(String),

    #[error("A value is out of range: {0}")]
    RangeError(String),

    #[error("Required data is missing: {0}")]
    MissingDataError(String),

    #[error("I/O error during code generation")]
    IoError(#[from] IoError),

    #[error("Empty coordinates")]
    EmptyCoordinates,
}

pub type GerberResult<T> = Result<T, GerberError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_msg() {
        let err = GerberError::CoordinateFormatError("Something went wrong".into());
        assert_eq!(
            err.to_string(),
            "Bad coordinate format: Something went wrong"
        );
    }
}
