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
use crate::{config as cfg, control};
use crate::{data, log_func};
use colored::Colorize;
use config::Config;
use serde::{self, Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use std::result::Result;
use std::sync;

pub const NUM_CMDS_NOPARAS: usize = 9;
pub const CMDNAMES: [&str; 12] = [
    "poweroff", 
    "reset",
    "scan",
    "report",
    "update",
    "sync",
    "list_reset",
    "list_mode",
    "init",
    "setinput", //with paras
    "list_length", // with paras
    "direct_spi", //with paras
];

#[allow(unused)]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct DataPacket {
    pub command_name: CommandTypes,
    pub paras: Option<Paras>,
    /// default: "0", from_input: "1", from other types: "2", from MQTT: complicated
    pub request_id: String,
}

impl Default for DataPacket {
    /// read from cfg.toml, or else return the default inputs.
    /// request id is 0 when default
    fn default() -> Self {
        if let Ok(c) = Config::builder()
            .add_source(config::File::with_name("cfg.toml"))
            .build()
        {
            return c
                .try_deserialize::<DataPacket>()
                .expect("Deserialization failed!");
        }

        eprintln!(
            ":{}",
            "cfg.toml is not configured correctly\nbuiltin inputs are used\n"
                .on_bright_red()
                .bold()
        );

        DataPacket {
            command_name: CommandTypes::SetInput,
            paras: Some(Paras::new(0f64, 0f32, 0)),
            request_id: "0".to_string(),
        }
    }
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

impl From<cfg::Input> for DataPacket {
    /// from , request_id = 1
    fn from(value: cfg::Input) -> Self {
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
        cfg::show()
    }
}

pub(crate) async fn wait4response_show() {
    if mcu_response().await {
        cfg::show()
    }
}

pub unsafe fn try_send(encoded: String) -> Result<(), DDSError> {
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

pub(crate) fn send_datapkg(pkg: DataPacket) -> Result<(), DDSError> {
    log_func!(cyan: "receiving DataPacket struct...");
    if let Ok(encoded) = serde_json::to_string_pretty(&pkg) {
        println!("\tdatapacket {pkg} => {:?}, ", encoded);
        unsafe {
            match try_send(encoded) {
                Ok(_) => {
                    log_func!("\tbytes Sent!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("\tsend error.{:?}", e);
                    Err(e)
                }
            }
        }
    } else {
        log_func!(on_red:"failed to encode datapacket to string");
        Err(DDSError::ConvertionError)
    }
}

pub(crate) fn send_cmd(cmd: CommandTypes) -> Result<(), DDSError> {
    log_func!(cyan: "receiving CommandTypes");
    // suitable for future `SetInput(Paras)`
    let datapkg = quick_cmd2data(cmd);
    send_datapkg(datapkg)
}

macro_rules! match_cmd {
    ($pattern:ident) => {
        Ok(DataPacket {
            command_name: CommandTypes::$pattern,
            paras: None,
            request_id: "2".to_string(),
        })
    };
}

macro_rules! match_allcmds {
    ($cmd:expr => $($pattern:ident,)+ ) => {
        match *$cmd {
            $(
                CommandTypes::$pattern => match_cmd!($pattern),
            )+
            _ => {
            log_func!(on_red:"doest not support with-paras-commands (fix in next edition)");
            Err(DDSError::IllegalArgument)  }
        }
    };

}

pub(crate) fn quick_cmd2data(cmd: CommandTypes) -> DataPacket {
    match cmd {
        // future: CommandTypes::SetInput(paras)
        CommandTypes::ListLength(ll) => DataPacket {
            command_name: CommandTypes::ListLength(ll),
            paras: None, // TODO
            request_id: "2".to_string(),
        },
        cmdtypes => DataPacket {
            command_name: cmdtypes,
            paras: None,
            request_id: "2".to_string(),
        },
    }
}

#[deprecated(since = "0.1.4", note = "use macro-based version instead")]
pub(crate) fn _quick_cmd2data_without_paras(cmd: &CommandTypes) -> Result<DataPacket, DDSError> {
    /// an easy way to get DataPacket from the given CommandTypes
    match *cmd {
        CommandTypes::PowerOff => match_cmd!(PowerOff),
        CommandTypes::Report => match_cmd!(Report),
        CommandTypes::Scan => match_cmd!(Scan),
        CommandTypes::Update => match_cmd!(Update),
        CommandTypes::DirectSPI => match_cmd!(DirectSPI),
        CommandTypes::Init => match_cmd!(Init),
        CommandTypes::ListMode => match_cmd!(ListMode),
        CommandTypes::ListReset => match_cmd!(ListReset),
        CommandTypes::Reset => match_cmd!(Reset),
        CommandTypes::Sync => match_cmd!(Sync),
        CommandTypes::SetInput | CommandTypes::ListLength(_) => {
            log_func!(on_red:"doest not support with-paras-commands (fix in next edition)");
            Err(DDSError::IllegalArgument)
        }
    }
}

pub(crate) fn quick_cmd2datapkg_no_paras(cmd: &CommandTypes) -> Result<DataPacket, DDSError> {
    match_allcmds!(
        cmd => PowerOff, Report, Scan, Update, DirectSPI, Init, ListMode, ListReset, Reset, Sync,
        SetInput,
    )
}

/// literal str to cmds (forbidden: bound variable strs)
macro_rules! match_str_cmds {
    ($cmd:expr; $($s:literal,)* => $($c:ident,)*) => {
        match $cmd {
            $(
                $s => Ok(CommandTypes::$c),
            )*
            _ => Err(DDSError::MissingArgumentError),
        }
    };
}

pub(crate) fn str2cmd(cmdstr: &str) -> Result<CommandTypes, DDSError> {
    // match cmdstr {
    //     "poweroff" => Ok(CommandTypes::PowerOff),
    //     "report" => Ok(CommandTypes::Report),
    //     "scan" => Ok(CommandTypes::Scan),
    //     "update" => Ok(CommandTypes::Update),
    //     "directspi" => Ok(CommandTypes::DirectSPI),
    //     "init" => Ok(CommandTypes::Init),
    //     "listmode" => Ok(CommandTypes::ListMode),
    //     "listreset" => Ok(CommandTypes::ListReset),
    //     _ => Err(DDSError::IllegalArgument),
    // }
    match_str_cmds!(cmdstr;
        "poweroff", "report", "scan", "update", 
        "directspi", "init", "listmode", "listreset", "sync", => PowerOff, Report, Scan, Update, DirectSPI, Init, ListMode, ListReset, Sync,)
}
/// an easy way to send command from command name (as &str)
/// does not support commands with paras
pub(super) fn quick_send(cmdstr: &str) -> Result<(), DDSError> {
    match cmdstr.try_into() {
        Ok(packet) => {
            send_datapkg(packet);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
