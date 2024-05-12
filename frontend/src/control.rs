//! the module to controlling the *frontend*
//! ,generating data packet, and communication to MCU
//! including the majority of the system logic.
#![allow(unused_variables, unused_features, unused_imports, dead_code)]

use crate::config::{CommandTypes, DDSInput};
use crate::data::{quick_cmd2datapkg_no_paras, quick_send_noparas, send_datapkg, send_msg};
use crate::nets;
use colored::Colorize;
use core::panic;
use std::sync::Once;
use std::{env, sync::Arc};

// use std::error::Error;
use crate::{
    cli::{CommunicationMethod, RunnerArgs},
    config::{quick_input_watcher, MCU, MCU_SOLID},
    ddserror::DDSError,
};

use std::time::Duration;

use std::{process::exit, thread};

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
    ($c:ident) => {
        println!("[{}] is done", $crate::func!().$c().bold());
    };
    ($($msg:literal),+) => {
        print!("[{}] ", $crate::func!().green().bold());
        $(
            print!("{}", ($msg).purple());
        )*
        println!();
    };

    ($($msg:expr),+) => {
        print!("[{}] ", $crate::func!().green().italic());
        $(
            print!("{:?}", ($msg));
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

#[macro_export]
macro_rules! prompt {
    () => {
        print!("{}", "$ ".on_blue());
    };
}

// TODO: read from env!
static mut HAS_SETUP: &bool = &false;
static ONCE_SETUP: Once = Once::new();

static mut HAS_CHECKED: &bool = &false;
static ONCE_CHECKED: Once = Once::new();

static mut REPL_ENABLED: &bool = &false;
static ONCE_ENABLE_REPL: Once = Once::new();

/// TODO: add a thread-safe to multi-changes it
static ONCE_CONNECTED: Once = Once::new();
static ONCE_DISCONNECTED: Once = Once::new();
static mut HAS_CONNECTED: &bool = &false;

pub(crate) static mut MODE: CommunicationMethod = CommunicationMethod::Wifi;

const VARS: [&str; 4] = [
    "DDSC_HAS_INIT",
    "DDSC_SUCCESSFULLY_CHECKED",
    "DDSC_REPL_ENABLED",
    "DDSC_HAS_CONNECTED",
];

unsafe fn get_mode() -> CommunicationMethod {
    MODE.clone()
}
/// **Assumption**: the DDSC_{}s are unique
unsafe fn once_setup() {
    ONCE_SETUP.call_once(|| {
        if env::var_os(VARS[0]).is_none() {
            env::set_var(VARS[0], "true");
        }
        HAS_SETUP = &true;
        log_func!();
    })
}

unsafe fn once_successfully_checked() {
    ONCE_CHECKED.call_once(|| {
        if env::var_os(VARS[1]).is_none() {
            env::set_var(VARS[1], "true");
        }
        HAS_CHECKED = &true;
        log_func!();
    })
}

unsafe fn once_repl() {
    ONCE_ENABLE_REPL.call_once(|| {
        if env::var_os(VARS[2]).is_none() {
            env::set_var(VARS[2], "true");
        }
        REPL_ENABLED = &true;
        log_func!();
    });
    // enter the repl mode:
    // TODO draw prompt,
    prompt!();
    // TODO wait for input
    // TODO parse the input and generate the message
    // TODO send msg
    log_func!();
}

#[allow(dead_code)]
unsafe fn _once_set_connected_via_env() {
    ONCE_CONNECTED.call_once(|| {
        HAS_CONNECTED = &true;
    })
}

#[allow(dead_code)]
unsafe fn _once_set_disconnected_via_env() {
    ONCE_DISCONNECTED.call_once(|| {
        HAS_CONNECTED = &false;
    })
}

static ONCE_CLEAR: Once = Once::new();

unsafe fn once_clear() {
    ONCE_CLEAR.call_once(|| {
        HAS_SETUP = &false;
        HAS_CHECKED = &false;
        HAS_CONNECTED = &true;
    })
}

/// WARN unsafe && too costy
/// considering the ~~trailing~~ *Jj*data racing** of
/// the `static mut`
#[warn(dead_code)]
unsafe fn set_connected() {
    if env::var_os(VARS[3]).is_none() {
        env::set_var(VARS[3], "true");
    }
    HAS_CONNECTED = &true;
}

/// WARN UNSAFE && too costy
#[warn(dead_code)]
unsafe fn set_disconnected() {
    if env::var_os(VARS[3]).is_none() {
        env::set_var(VARS[3], "true");
    }
    HAS_CONNECTED = &false;
}

#[allow(unused)]
pub fn has_setup() -> bool {
    if env::var_os(VARS[0]).is_none() {
        return false;
    }
    true
}
pub fn has_checked() -> bool {
    if env::var_os(VARS[1]).is_none() {
        return false;
    }
    true
}
pub fn has_enable_repl() -> bool {
    if env::var_os(VARS[1]).is_none() {
        return false;
    }
    true
}
pub const fn has_connected() -> bool {
    unsafe { *HAS_CONNECTED }
}

pub(crate) fn repl() {
    unsafe {
        if !has_enable_repl() {
            log_func!(red:"REPL had enable already");
        } else {
            once_repl();
        }
    }
    log_func!();
}

pub(crate) fn init_system() {
    let climode = unsafe { get_mode() };

    connect2esp32(&climode);

    unsafe {
        once_setup();
    }
    checks();

    log_func!();
}

fn connect2esp32(climode: &CommunicationMethod) {
    // fn connect2esp32(mcu: &MCU, climode: &CommunicationMethod) {
    match climode {
        CommunicationMethod::Ble => {
            log_func!(on_bright_magenta:"\tConnecting to ESP32 via BLE");
        }

        CommunicationMethod::Iot => {
            log_func!(on_bright_magenta:"\tConnecting to ESP32 via IoT platform");
        }
        CommunicationMethod::Wifi => {
            log_func!(on_bright_magenta:"\tConnectingto ESP32 via Wifi");
            // let (retry_times, retry_int) = mcu.retry_settings();
            let (retry_times, retry_int) = MCU_SOLID.retry_settings();
            let mut loops = 1;
            try_connect(retry_times, retry_int);
            // while let Err(e) = try_connect(retry_times, retry_int) {
            //     if loops > retry_times {
            //         log_func!(on_red:"Closed.");
            //         exit(exitcode::UNAVAILABLE);
            //     }
            //     eprintln!(
            //         "{loops}/{retry_times} tries. failed to connect to {}",
            //         MCU_SOLID.ip()
            //     );
            //     eprintln!("reconnect in {} seconds", retry_int);
            //     thread::sleep(Duration::from_secs_f32(retry_int));
            //     loops += 1;
            // }
        }
        CommunicationMethod::Wired => {
            log_func!(on_bright_magenta:"\t Established GPIO connection to ESP32");
        }
    }

    log_func!();
}

// TODO finish the DDSError

fn try_connect(re_time: u32, re_int: f32) -> Result<(), DDSError> {
    let Ok(_) = nets::client_connect() else {
        log_func!(on_red:"Connection tried failed");
        return Err(DDSError::ConnectionLost);
    };

    unsafe {
        set_connected();
    }

    Ok(())
}

fn disconnect() {
    unsafe {
        if has_connected() {
            set_disconnected();
        }
    }
    log_func!();
}

fn checks() {
    //

    if !has_setup() {
        log_func!(red:"Havn't init!");
    } else if has_checked() {
        log_func!(red:"Checked Already.Checked Again");
    } else {
        check_esp32();
        unsafe {
            once_successfully_checked();
        }
    }

    log_func!();
}

fn check_esp32() {
    send_datapkg(quick_cmd2datapkg_no_paras(&CommandTypes::Report).unwrap());
    log_func!();
}

/// Duration is unneeded
/// NOTICE: 我们将 wait 放在frontend 处理，只是为了方便点。
/// 所以实际上poweroff wait 是: wait 之后再传数据
pub(crate) fn poweroff(wait: Option<u64>) {
    if let Some(ms) = wait {
        thread::sleep(Duration::from_millis(ms));
    }
    // send_msg(msg_json);
    quick_send_noparas("poweroff").expect("failed to poweroff! ");

    // TODO:
    unsafe {
        once_clear();

        disconnect();
    }
    log_func!("power off");
}

pub(super) fn run(args: RunnerArgs) {
    let script = args.instruction_input;
    let mode = args.mode;
    unsafe {
        MODE = mode;
    }
    quick_input_watcher(script);
    log_func!();
}

pub(crate) fn execute(script: String) {
    if !has_setup() {
        log_func!(red:"HASN'T init!");
        init_system();
    }
    // println!("executing...{}", &script.blue());

    // raw_execute(script);

    // log_func!(" script executed.");
}

fn raw_execute(script: String) {
    log_func!();
}

pub(crate) fn monitor() {
    // TODO : open serial port
    todo!()
}

//TODO change the AddrParseError -> DDSError
/// bind esp32 using esp32's IP?
/// 1. read and load the config
/// as for trying to connect... this is the mission of another function
#[deprecated(
    since = "0.1.8",
    note = "use the static ref `MCU_SOLID` instead, which is a global static variable"
)]
pub(crate) fn try_parse_mcu() -> Result<MCU, DDSError> {
    match MCU::new() {
        Ok(mcu) => {
            if mcu.debug() {
                println!("bind success, {}", "debug mode: On".cyan().bold());
                println!("trying to connect to ...{} ", mcu.ip());
                println!("<{}> -- <{}>", mcu.device_id(), mcu.device_id());
                println!("<{}> -- <{}>", mcu.device_secret(), mcu.device_secret());
            } else {
                println!("{}", "debug mode: Off".cyan().italic());
            }
            Ok(mcu)
        }
        Err(e) => Err(e),
    }
}

// --------------------
