// use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::fmt::Display;

use crate::cli;

/// fault error
#[derive(Debug)]
pub enum DDSError {
    ConnectionLost,
    Forbidden,
    NoTarget,
    #[doc(hidden)]
    #[cfg(feature = "failpoints")]
    FailPoint,
}

impl Display for DDSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub(crate) fn repl() {}

pub(crate) fn init_system(_mode: cli::CommunicationMethod) {
    check_esp32();
    // ESP32: init_dds(); //init_dds() is done by esp32
}

fn _connect2esp32(mode: cli::CommunicationMethod) {
    while let Err(e) = _try_connect(mode) {
        eprintln!("failed to connect! {}", e);
    }
}

fn _try_connect(_mode: cli::CommunicationMethod) -> Result<i32, DDSError> {
    Ok(1)
}
fn check_esp32() {}

pub(crate) fn send_msg(encoded: String) {
    println!("[Transfer Emulator]: {}", encoded);
    let decoded = json!(encoded);
    println!("[Transfer Decoder]: {}, Sent!", decoded);
}
