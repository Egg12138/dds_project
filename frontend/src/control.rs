use colored::Colorize;
use config::ConfigError;
use core::panic;
use std::env;
use std::sync::Once;
// use std::error::Error;
use crate::{
    cli::{self, CommunicationMethod, RunnerArgs},
    config::{quick_input_watcher, MCU},
};
use serde_json::json;
use std::{fmt::Display, process::exit, thread, time::Duration};

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

// TODO: read from env!
static mut HAS_INIT: &bool = &false;
static ONCE_INIT: Once = Once::new();

static mut HAS_CHECKED: &bool = &false;
static ONCE_CHECKED: Once = Once::new();

static mut REPL_ENABLED: &bool = &false;
static ONCE_ENABLE_REPL: Once = Once::new();

/// TODO: add a thread-safe to multi-changes it
static ONCE_CONNECTED: Once = Once::new();
static ONCE_DISCONNECTED: Once = Once::new();
static mut HAS_CONNECTED: &bool = &false;

static mut MODE: CommunicationMethod = CommunicationMethod::Wifi;

const VARS: [&str; 4] = [
    "DDSC_HAS_INIT",
    "DDSC_SUCCESSFULLY_CHECKED",
    "DDSC_REPL_ENABLED",
    "DDSC_HAS_CONNECTED",
];

/// **Assumption**: the DDSC_{}s are unique
unsafe fn once_init() {
    ONCE_INIT.call_once(|| {
        if env::var_os(VARS[0]).is_none() {
            env::set_var(VARS[0], "true");
        }
        HAS_INIT = &true;
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
    })
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
        HAS_INIT = &false;
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
pub fn has_init() -> bool {
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
pub fn enable_repl() -> bool {
    if env::var_os(VARS[1]).is_none() {
        return false;
    }
    true
}
pub const fn has_connected() -> bool {
    unsafe { *HAS_CONNECTED }
}

/// fault error
#[allow(unused)]
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

pub(crate) fn repl() {
    unsafe {
        if !REPL_ENABLED {
            log_func!(red:"REPL had enable already");
        } else {
            once_repl();
        }
    }
    log_func!();
}

pub(crate) fn init_system() {
    // unsafe {
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

    while let Err(e) = try_connect() {
        eprint!("[ERROR]:{:?}", e);
        eprintln!("read the mcu config again...");
    }

    unsafe {
        once_init();
    }
    connect2esp32();
    checks();
    // }

    log_func!();
}

fn connect2esp32() {
    while let Err(e) = try_connect() {
        eprintln!("failed to connect! {}", e);
    }

    log_func!();
}

// TODO finish the DDSError
fn try_connect() -> Result<i32, DDSError> {
    Ok(1)
}

fn checks() {
    //

    if !has_init() {
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
    send_msg("check esp32".to_string());
    log_func!();
}

pub(crate) fn send_msg(encoded: String) {
    log_func!(cyan: "receiving...");
    println!("{}", encoded);

    unsafe {
        let decoded = match (MODE) {
            CommunicationMethod::Ble => {
                log_func!(on_bright_magenta:"Sending via Ble");
                json!(encoded)
            }
            CommunicationMethod::Iot => {
                log_func!(on_bright_magenta:"Sending via Wifi to IoT platform");
                json!(encoded)
            }
            CommunicationMethod::Wifi => {
                log_func!(on_bright_magenta:"Sending via Wifi to ESP32");
                json!(encoded)
            }
            CommunicationMethod::Wired => {
                log_func!(on_bright_magenta:"Sending via GPIO connection to ESP32");
                json!(encoded)
            }
        };
        println!("{}, Sent!", decoded);
    }

    log_func!(cyan: "sending...");
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

    unsafe {
        once_clear();
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
    if let Ok(mcucfg) = MCU::new() {
        let mode: CommunicationMethod = mcucfg.mode();
        if !has_init() {
            log_func!(red:"HASN'T init!");
            init_system();
        }
        println!("executing...{}", &script.blue());

        raw_execute(script);

        log_func!(" script executed.");
    } else {
        panic!();
    }
}

fn raw_execute(script: String) {
    log_func!();
}

pub(crate) fn monitor() {
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
