//! re-write some command functions
use crate::data::*;
use crate::ddserror::DDSError;
use crate::log_func;
use crate::{config::*, data};
use colored::Colorize;
use config::{Config, File};
use serde_json;

type Channel = (u8, u8, u8, u8);
impl From<u8> for Channel {
    /// from value: 1011 -> Channel (1, 0, 1, 1) MSB
    fn from(value: u8) -> Self {
        let ch0 = (value & 0b1000);
        let ch1 = (value & 0b0100);
        let ch2 = (value & 0b0010);
        let ch3 = (value & 0b0001);
        (ch0, ch1, ch2, ch3)
    }
}

macro_rules! template_noparas_cmd {
    ($c:ident) => {
        let $c = str2cmd(stringify!($c)).unwrap();
        let Ok(_) = send_datapkg(quick_cmd2data($c)) else {
            log_func!(on_red: "failed to send datapkg");
            return;
        };
        log_func!();
    };
}
pub fn reset_dds() {
    // reset is definitely no error
    template_noparas_cmd!(reset);
}

pub fn poweroff_dds() {
    template_noparas_cmd!(poweroff);
}

pub fn update_dds() {
    template_noparas_cmd!(update);
}

pub fn listmode_dds() {
    template_noparas_cmd!(listmode);
}

pub fn sync_dds() {
    template_noparas_cmd!(sync);
}

// with-paras cmds
// TODO: important, refactor from `Input` driven into the common DataPacket driven.
pub fn setinput_dds() -> Result<(), DDSError> {
    let builder = Config::builder()
        .add_source(File::with_name(LOCAL_CFG_PATH))
        .build();
    match builder {
        Ok(paras) => match paras.try_deserialize::<Input>() {
            Ok(input) => {
                let datapkg = DataPacket::from(input);
                log_func!(on_bright_cyan:"41");
                send_datapkg(datapkg)
            }
            Err(e) => {
                log_func!(on_bright_cyan:"46");
                Err(e.into())
            }
        },
        Err(e) => {
            log_func!(on_red:"failed to build config");
            Err(e.into())
        }
    }
}

pub fn CSR(channel: Channel) {}
