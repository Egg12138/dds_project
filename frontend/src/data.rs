//! `data` module: data communication and data packaging..
//! data format:
//! example:
//! ```JSON
//! {
//!  "command_name": "set_dds",
//! "paras" :  {
//!  "freq_hz": 10000.22"// the required DDS output frequency,
//!  "volt_mv": 1145.14 // the required DDS output voltage,
//!  "phs_oft": 180     // the required DDS output phase offset,
//!  "wave" : "sin"
//! }
//! }
//! ```
//!
//!

use crate::cli::{CommunicationMethod, DataArgs};
use crate::config::{CommandTypes, Input, Paras};
use crate::control::{has_connected, poweroff};
use crate::ddserror::{self, DDSError};
use crate::log_func;
use crate::{config, control};
use colored::Colorize;
use serde::{self, Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use std::result::Result;
use std::sync;

pub const CMDNAMES: [&str; 12] = [
    "poweroff",
    "reset",
    "setinput",
    "scan",
    "report",
    "update",
    "direct_spi",
    "sync",
    "list_reset",
    "list_mode",
    "list_length",
    "init",
];

#[allow(unused)]
#[derive(Debug, Deserialize, Serialize)]
pub struct DataPacket {
    command_name: CommandTypes,
    paras: Option<Paras>,
    request_id: String,
}

impl Display for DataPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            json!({
                "command_name": self.command_name,
                "paras": self.paras,
                "request_id": self.request_id
            })
            .to_string()
            .bright_blue()
        )
    }
}

impl TryFrom<&str> for DataPacket {
    type Error = DDSError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !CMDNAMES.contains(&value) {
            eprintln!("{value} is not a valid command_name");
            Err(DDSError::IllegalArgument)
        } else if value == "scan" || value == "setinput" {
            log_func!(on_red:"does not support with-paras-command setinput/ramp .");
            Err(DDSError::ConvertionError)
        } else {
            Ok(DataPacket {
                command_name: value.into(),
                paras: None,
                request_id: "2".to_string(),
            })
        }
    }
}

impl From<config::Input> for DataPacket {
    fn from(value: config::Input) -> Self {
        let command_name = value.command_name();
        let freq_hz = value.freq();
        let vol_mv = value.vol();
        let ph_oft = value.phase();
        DataPacket {
            command_name: command_name.clone(),
            paras: Some(Paras::new(freq_hz, vol_mv, ph_oft)),
            request_id: "1".to_string(),
        }
    }
}

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

unsafe fn try_send(encoded: String) -> Result<(), DDSError> {
    log_func!(purple:"trying to send...");
    println!("\t{}", encoded);
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
    log_func!(cyan: "receiving String...");

    print!("\t{msg} => {:?}, ", msg);

    unsafe {
        match try_send(msg) {
            Ok(_) => println!("\tbytes Sent!"),
            Err(e) => eprintln!("\tsend error.{:?}", e),
        }
    }

    log_func!(cyan: " sent.");
}

pub(crate) fn send_datapkg(pkg: DataPacket) {
    log_func!(cyan: "receiving DataPacket struct...");
    if let Ok(encoded) = serde_json::to_string_pretty(&pkg) {
        println!("\tdatapacket {pkg} => {:?}, ", encoded);
        unsafe {
            match try_send(encoded) {
                Ok(_) => println!("\tbytes Sent!"),
                Err(e) => eprintln!("\tsend error.{:?}", e),
            }
        }
    } else {
        panic!("Data packet cannot be serialized!!")
    }

    log_func!(cyan: " sent.");
}

pub(super) fn quick_cmd2data(cmd: &CommandTypes) -> Result<DataPacket, &str> {
    match *cmd {
        CommandTypes::PowerOff => Ok(DataPacket {
            command_name: CommandTypes::PowerOff,
            paras: None,
            request_id: "2".to_string(),
        }),
        CommandTypes::Report => Ok(DataPacket {
            command_name: CommandTypes::Report,
            paras: None,
            request_id: "2".to_string(),
        }),
        CommandTypes::Scan => Ok(DataPacket {
            command_name: CommandTypes::Scan,
            paras: None,
            request_id: "2".to_string(),
        }),

        _ => {
            log_func!(on_red:"does not support with-paras-commands setinput/ramp .");
            Err("error")
        }
    }
}

pub(super) fn quick_send(cmd: &str) -> Result<(), DDSError> {
    match cmd.try_into() {
        Ok(packet) => {
            send_datapkg(packet);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
