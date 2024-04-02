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

use std::fmt::Display;
#[allow(unused)]
#[derive(Debug)]
pub enum DDSError {
    ConfigError,
    AddressParseError,
    SendingError,
    MissingArgumentError,
    ConnectionLost,
    Forbidden,
    NoTarget,
    Task,
    #[doc(hidden)]
    #[cfg(feature = "failpoints")]
    FailPoint,
}

impl Display for DDSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
