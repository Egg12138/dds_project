//! Error module for this project
//! which contains: (drafat)
//! * ConfigError
//! * AddressParseError
//! * SendingError
//! * MissingArgumentError
//! * ConnectionLostError
//! * Forbidden?
//! * NoTargetError?
//! * etc.

use std::error::Error;
use std::fmt::Display;
use std::io;
use std::net;
#[allow(unused)]
#[derive(Debug)]
pub enum DDSError {
    ConfigError(config::ConfigError),
    AddressParseError(net::AddrParseError),
    SendingError,
    IO(io::Error),
    MissingArgumentError,
    ConnectionLost,
    Forbidden,
    NoTarget,
    IllegalArgument,
    ConvertionError,
    #[doc(hidden)]
    #[cfg(feature = "failpoints")]
    FailPoint,
}

impl Display for DDSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DDSError {}

impl From<config::ConfigError> for DDSError {
    fn from(value: config::ConfigError) -> Self {
        DDSError::ConfigError(value)
    }
}

impl From<net::AddrParseError> for DDSError {
    fn from(value: net::AddrParseError) -> Self {
        DDSError::AddressParseError(value)
    }
}

impl From<io::Error> for DDSError {
    fn from(value: io::Error) -> Self {
        DDSError::IO(value)
    }
}
