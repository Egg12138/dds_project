use colored::Colorize;
use config::ConfigError;
// use std::error::Error;
use crate::{cli, config::MCU};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::any::type_name;
use std::{fmt::Display, net::AddrParseError, process::exit, str::FromStr, thread, time::Duration};

#[macro_export]
macro_rules! func {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}

#[macro_export]
macro_rules! log_func {
    () => {
        println!("[{}] is done", $crate::func!().green().bold());
    };
($($msg:literal),+) => {
        print!("[{}] ", $crate::func!().green().bold());
        $(
            print!("{}", ($msg).purple());
        )*
        println!();

    };
    ( $c:ident : $($msg:literal),*)  => {
        print!("[{}] ", $crate::func!().$c().bold());
        $(
            print!("{}", ($msg).purple());
        )*
        println!();
    };



}

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

pub(crate) fn init_system(mode: cli::CommunicationMethod) {
    if let Err(e) = try_parse_mcu() {
        //TODO if failed to parse.
        eprintln!(
            "{}:\n{:?}",
            "[control::try_parse_mcu] failed to parse, exit".on_red(),
            e
        );
        eprintln!("current directory is {:?}", std::env::current_dir());
        exit(0x01)
    }

    while let Err(e) = try_connect(mode) {
        eprint!("[ERROR]:{:?}", e);
        eprintln!("read the mcu config again...");
    }

    log_func!();
}

fn _connect2esp32(mode: cli::CommunicationMethod) {
    while let Err(e) = try_connect(mode) {
        eprintln!("failed to connect! {}", e);
    }

    log_func!();
}

// TODO finish the DDSError
fn try_connect(_mode: cli::CommunicationMethod) -> Result<i32, DDSError> {
    Ok(1)
}
fn check_esp32() {
    log_func!();
}

pub(crate) fn send_msg(encoded: String) {
    log_func!(cyan: "receiving...");
    println!("{}", encoded);
    let decoded = json!(encoded);
    log_func!(cyan: "sending...");
    println!("{}, Sent!", decoded);
}

/// Duration is unneeded
/// NOTICE: 我们将 wait 放在frontend 处理，只是为了方便点。
/// 所以实际上poweroff wait 是: wait 之后再传数据
pub(crate) fn poweroff(wait: Option<u64>) {
    let msg_json = "{}".to_owned();
    if let Some(ms) = wait {
        thread::sleep(Duration::from_millis(ms));
    }
    send_msg(msg_json);

    log_func!("power off");
}

pub(crate) fn execute(script: String) {
    log_func!(" script executed.");
}

pub(crate) fn serial_monitor() {
    // TODO : open serial port
    // NOTICE: 不是波形显示器！只是串口监视器&参数显示器
    // NOTICE: 不推荐用这个,用外置的串口监视器更好点,无线连接就只参数显示器
    todo!()
}

//TODO change the AddrParseError -> DDSError
/// bind esp32 using esp32's IP?
/// 1. read and load the config
/// as for trying to connect... this is the mission of another function
pub(crate) fn try_parse_mcu() -> Result<MCU, ConfigError> {
    match MCU::new() {
        Ok(mcu) => {
            if mcu.debug() {
                println!("bind success, {}", "debug mode: On".cyan().bold());
                println!("trying to connect to ...{} ", mcu.ip());
                println!("<{}> -- <{}>", mcu.pub_key(), mcu.pub_key());
                println!("<{}> -- <{}>", mcu.privt_key(), mcu.privt_key());
            } else {
                println!("{}", "debug mode: Off".cyan().italic());
            }
            Ok(mcu)
        }
        Err(e) => Err(e),
    }
}
