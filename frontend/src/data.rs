//! `data` module: data communication and data packaging..
//! data format:
//! example:
//! ```JSON
//! {
//!  "type": "set_dds",
//!  "freq_hz": 10000.22"// the required DDS output frequency,
//!  "volt_mv": 1145.14 // the required DDS output voltage,
//!  "phs_oft": 180     // the required DDS output phase offset,
//!  "wave" : "sin"
//! }
//! ```

use crate::cli::CommunicationMethod;
use crate::control::has_connected;
use crate::ddserror::DDSError;
use crate::log_func;
use crate::{config, control};
use colored::Colorize;
use serde::{self};
use serde_json::json;
use std::sync;
/// wait for the response from ESP32
async fn mcu_response() -> bool {
    //TODO: using Notify.rs
    true
}

/// pass op: FnOnce
#[allow(unused)]
async fn wait4response_and() {
    // pseudo code:
    if mcu_response().await {
        config::show()
    }
}

pub(crate) async fn wait4response_show() {
    if mcu_response().await {
        config::show()
    }
}

/// the standard/universal packaging dataformat
fn packaging(data_string: String) -> serde_json::Value {
    log_func!(purple:" packet finished");
    json!(data_string)
}

unsafe fn try_send(encoded: serde_json::Value) -> Result<(), DDSError> {
    log_func!(purple:"trying to send...");
    println!("\t{:?}", encoded);

    unsafe {
        match control::MODE {
            CommunicationMethod::Ble => {
                log_func!(on_bright_magenta:"\tSending via Ble");
            }
            CommunicationMethod::Iot => {
                log_func!(on_bright_magenta:"\tSending via Wifi to IoT platform");
            }
            CommunicationMethod::Wifi => {
                log_func!(on_bright_magenta:"\tSending via Wifi to ESP32");
            }
            CommunicationMethod::Wired => {
                log_func!(on_bright_magenta:"\tSending via GPIO connection to ESP32");
            }
        }
    }

    if !has_connected() {
        log_func!(on_red:" havn't connected to MCU!");
        Err(DDSError::ConnectionLost)
    } else {
        log_func!(magenta:" sent!");
        Ok(())
    }
}

pub(crate) fn send_msg(msg: String) {
    log_func!(cyan: "receiving...");
    print!("\tmsg => {}, ", msg);

    let packet = packaging(msg);

    unsafe {
        match try_send(packet) {
            Ok(_) => println!("\tbytes Sent!"),
            Err(e) => eprintln!("\tsend error.{:?}", e),
        }
    }

    log_func!(cyan: " sent.");
}
