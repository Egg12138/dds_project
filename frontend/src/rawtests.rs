//! re-write some command functions
use crate::config::*;
use crate::data::*;
use crate::ddserror::DDSError;
use crate::log_func;
use crate::register_controller::*;
use colored::Colorize;
use config::{Config, File};

#[test]
fn read_config_works() {
    assert!(setinput_dds().is_ok())
}

#[test]
fn set_input_works() {
    let mut input = Input::default();
    let freq = 114514.3;
    let vol = 3000f32;
    let phase = 90;
    input.set_input(freq, vol, phase);
    if let Ok(msg) = serde_json::to_string(&input) {
        println!("\t\t serde_json::tostring(input) is into {msg}");
    }
}

#[test]
fn data_packaging_from_input() {
    let input = Input::from((114513.3, 3000_f32, 90));
    let datapackage = DataPacket::from(input);
}

#[test]
fn cmd2datapkg() {
    let poweroff = CommandTypes::PowerOff;
    if let Ok(datapacket) = quick_cmd2datapkg_no_paras(&poweroff) {
        println!("{:?}", datapacket);
        send_datapkg(datapacket);
    }
}

#[test]
fn cmd2datastr() {
    let cmdstr = "report";
    let cmd = str2cmd(cmdstr);
    assert!(cmd.is_ok());
    let cmd = cmd.unwrap();
    let datapkg = quick_cmd2datapkg_no_paras(&cmd);
    assert!(datapkg.is_ok());
    assert_eq!(
        datapkg.unwrap(),
        DataPacket {
            command_name: CommandTypes::Report,
            paras: None,
            request_id: 2
        }
    );
}

#[test]
fn str2cmd_works() {
    let cmd = "illegal_cmd";
    assert!(TryInto::<DataPacket>::try_into(cmd).is_err())
}
#[test]
fn unsafe_try_send_str() {
    send_msg("TRY SENT".to_string());
    let input = Input::from((114514.3, 2333_f32, 150));
    unsafe {
        let input_str = serde_json::to_string(&input).unwrap_unchecked();
        if let Ok(_) = try_send(input_str) {
            log_func!(cyan:"sent");
        }
    }
}

#[test]
fn read_datapkg_from_cfg() {
    let input = Input::from_config(LOCAL_CFG_PATH);
    let datapkg = DataPacket::default();
    let datapkg_from_input = DataPacket::from(input);
    assert_ne!(datapkg, datapkg_from_input);
}
