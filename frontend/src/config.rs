//! the config.rs checks the validation of input arguments

#![allow(deprecated)]

use crate::control;
use crate::data;
use crate::ddserror::DDSError;
use crate::log_func;
use colored::Colorize;
use config::{Config, Environment, File};
use core::panic;
use lazy_static::lazy_static;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::fmt::write;
#[allow(unused_imports)]
use std::future::Future;
use std::net::Ipv4Addr;
use std::path::Path;
//use std::str::FromStr;
use std::sync::{mpsc::channel, RwLock};
use std::time::Duration;

pub const CFG_WATCH_INT: u64 = 2; // in seconds
/// the `dds input` local configuration TOML file in the {workspace},
pub const LOCAL_CFG_PATH: &str = "./config/cfg.toml";
/// mcu settings
pub const LOCAL_MCU_CFG_PATH: &str = "./config/mcucfg.toml";
pub const DEFAULT_MCU_CFG_PATH: &str = "/.config/default_mcgcfg.toml";
pub const ENV_PREFIX: &str = "dds";

lazy_static! {
    static ref CFG: RwLock<Config> = RwLock::new({
        let mut configuration = Config::default();
        configuration
            .merge(File::with_name(LOCAL_CFG_PATH))
            .unwrap();
        configuration
    });
}

/// Parse json field string as IPv4,
#[repr(C)]
#[derive(Debug, Deserialize)]
pub(crate) struct MCU {
    debug: bool,
    connection: Connection,
    iot: IoT,
}

#[repr(C)]
#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Connection {
    ip: Ipv4Addr,
    pwd: String,
    // mode: CommunicationMethod,
    retry: u32,
    /// in secs
    retry_interval: f32,
}

#[repr(C)]
#[allow(unused)]
#[derive(Debug, Deserialize)]
struct IoT {
    public_key: String,
    private_key: String,
}

#[allow(unused)]
impl MCU {
    pub(crate) fn new() -> Result<Self, DDSError> {
        let s = Config::builder()
            .add_source(File::with_name(LOCAL_MCU_CFG_PATH))
            .add_source(File::with_name(DEFAULT_MCU_CFG_PATH).required(false))
            // add in cfgs from the environment (with a prefix of DDS)
            .add_source(Environment::with_prefix(ENV_PREFIX))
            .build()?;
        println!("[MCU::new] debug: {:?}", s.get_bool("debug"));
        // println!(
        //     "[MCU::new] private key: {:?}",
        //     s.get::<String>("connection.ip")
        // );
        log_func!("private key", s.get::<String>("connection.ip"));
        log_func!("table: ", s.get_table("iot"));
        // println!("[MCU::new] table: {:?}", s.get_table("iot"));
        log_func!("done");
        s.try_deserialize().map_err(DDSError::ConfigError)
    }

    pub(crate) fn debug(&self) -> bool {
        self.debug
    }
    pub(crate) fn ip(&self) -> Ipv4Addr {
        self.connection.ip
    }
    // pub(crate) fn mode(&self) -> CommunicationMethod {
    // self.connection.mode
    // }

    #[deprecated]
    pub(crate) fn pub_key(&self) -> &String {
        &self.iot.public_key
    }

    #[deprecated]
    pub(crate) fn privt_key(&self) -> &String {
        &self.iot.private_key
    }

    pub(crate) fn keypais(&self) -> (&String, &String) {
        (&self.iot.public_key, &self.iot.private_key)
    }

    /// return : (retry_time, retry_int)
    pub(crate) fn retry_settings(&self) -> (u32, f32) {
        (self.connection.retry, self.connection.retry_interval)
    }

    pub(crate) fn pwd(&self) -> &String {
        &self.connection.pwd
    }

    pub(crate) fn change_pwd(&mut self, newone: String) {
        self.connection.pwd = newone;
    }

    pub(crate) fn reconnect(&self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandTypes {
    SetInput,
    PowerOff,
    Scan,
    Report, // check
    Reset,
    Update,
    DirectSPI,
    ListMode,
    ListLength(u32),
    ListReset,
    Sync,
    Init,
    // MemStorage(Vec<String>),
}

impl std::fmt::Display for CommandTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetInput => write!(f, "setinput"),
            Self::PowerOff => write!(f, "poweroff"),
            Self::Scan => write!(f, "scan"),
            Self::Report => write!(f, "report"),
            Self::Reset => write!(f, "reset"),
            Self::DirectSPI => write!(f, "direct SPI"),
            Self::ListLength(len) => write!(f, "set list length to {}", *len),
            Self::ListReset => write!(f, "reset list"),
            Self::ListMode => write!(f, "list mode"),
            Self::Sync => write!(f, "synchronized"),
            // Self::MemStorage(spicmds) => write!(f, "stored spi cmds {:?}", *spicmds),
            _ => panic!("Invalid command type"),
        }
    }
}

impl From<&str> for CommandTypes {
    fn from(value: &str) -> Self {
        match value {
            "setinput" => CommandTypes::SetInput,
            "poweroff" => CommandTypes::PowerOff,
            "scan" => CommandTypes::Scan,
            "report" => CommandTypes::Report,
            "reset" => CommandTypes::Reset,
            "direct_spi" => CommandTypes::DirectSPI,
            "list_mode" => CommandTypes::ListMode,
            "list_reset" => CommandTypes::ListReset,
            "sync" => CommandTypes::Sync,
            with_arg => {
                let split: Vec<&str> = with_arg.split(',').collect();
                match split[0] {
                    "list_length" => CommandTypes::ListLength(split[1].parse::<u32>().unwrap()),
                    _ => panic!("Unknown command type"),
                }
            }
        }
    }
}
/// a standard collection of data to be sent.
/// but `quick_Watcher` is a faster way to sending messages.
#[repr(C)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Input {
    pub command_name: CommandTypes,
    pub paras: Paras,
    // pub collect: bool,
}

#[repr(C)]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct Paras {
    freq_hz: f64,
    vol_mv: f32,
    ph_oft: u8,
}

impl Paras {
    pub(crate) fn new(freq_hz: f64, vol_mv: f32, ph_oft: u8) -> Self {
        Paras {
            freq_hz,
            vol_mv,
            ph_oft,
        }
    }
    pub(self) fn set(&mut self, f: f64, v: f32, p: u8) {
        self.freq_hz = f;
        self.vol_mv = v;
        self.ph_oft = p;
    }
}

impl Default for Input {
    /// read from cfg.toml, or else return the default inputs.
    fn default() -> Self {
        if let Ok(cfg) = Config::builder()
            .add_source(config::File::with_name("cfg.toml"))
            .build()
        {
            return cfg
                .try_deserialize::<Input>()
                .expect("Deserialization failed!");
        }

        eprintln!(
            ":{}",
            "cfg.toml is not configured correctly\nbuiltin inputs are used\n"
                .on_bright_red()
                .bold()
        );

        Input {
            command_name: CommandTypes::SetInput,
            paras: Paras {
                freq_hz: 0_f64,
                vol_mv: 0_f32,
                ph_oft: 0,
            },
        }
    }
}

impl From<(f64, f32, u8)> for Input {
    fn from(value: (f64, f32, u8)) -> Self {
        Input {
            command_name: CommandTypes::SetInput,
            paras: Paras {
                freq_hz: value.0,
                vol_mv: value.1,
                ph_oft: value.2,
            },
        }
    }
}
impl From<Config> for Input {
    fn from(value: Config) -> Self {
        value.try_deserialize().unwrap_or_else(|e| {
            eprintln!("{e}");
            log_func!(on_magenta:"config failed to deserialize, use the default settings");
            Self::default()
        })
    }
}

#[allow(unused)]
impl Input {
    #[deprecated]
    fn _syslevel_path() -> &'static str {
        #[cfg(target_family = "windows")]
        const SysPATH: &str = "%APPDATA%\\dds_controller\\cfg.toml";

        #[cfg(target_family = "unix")]
        const SysPATH: &str = "~/.config/dds_controller//cfg.toml";

        SysPATH
    }

    pub(crate) fn from_config(path: &str) -> Self {
        if let Ok(builder) = Config::builder().add_source(File::with_name(path)).build() {
            builder.try_deserialize().unwrap_or_default()
        } else {
            eprintln!("failed to build config from the {}", path);
            Input::default()
        }
    }

    pub(crate) fn set_input(&mut self, f: f64, v: f32, p: u8) {
        self.paras.set(f, v, p);
    }

    fn handle(&mut self) {
        let cfgmap = CFG.read().unwrap().clone();
        let input = Input::from(cfgmap);
        self.set_input(input.freq(), input.vol(), input.phase());
        self.command_name = input.command_name;

        println!(
            "* Input:: 
        \n\x1b[31m {:?}\x1b[0m",
            self
        );
        let encoded = serde_json::to_string(&self).unwrap_or_default();
        data::send_msg(encoded);
    }

    pub fn freq(&self) -> f64 {
        self.paras.freq_hz
    }
    pub fn vol(&self) -> f32 {
        self.paras.vol_mv
    }
    pub fn phase(&self) -> u8 {
        self.paras.ph_oft
    }
    pub fn command_name(&self) -> &CommandTypes {
        &self.command_name
    }
    // #[allow(unused)]
    pub fn watch_dds_input(&mut self) {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_secs(CFG_WATCH_INT)),
        )
        .unwrap();

        assert!(watcher
            .watch(Path::new(LOCAL_CFG_PATH), RecursiveMode::NonRecursive,)
            .is_ok());
        // TODO 1. loop, in another tty
        // TODO 2. the real time watch is opened when user enable
        // TODO 3. disable this associated function when `scan` is enable

        loop {
            match rx.recv() {
                Ok(Ok(Event {
                    kind: notify::event::EventKind::Modify(_),
                    ..
                })) => {
                    println!(" NEW input:");
                    assert!(CFG.write().unwrap().refresh().is_ok());
                    self.handle();
                }
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    break;
                }
                _ => {}
            }
        }

        log_func!(red:"unexpected RecvError!");
    }

    fn freq_valid(&self) -> bool {
        //TODO compared to the values read from `ddsinfo.toml`
        true
    }

    fn vol_valid(&self) -> bool {
        true
    }

    fn ph_valid(&self) -> bool {
        true
    }

    pub(crate) fn valid_input(&self) -> bool {
        self.freq_valid() && self.vol_valid() && self.ph_valid()
    }
}

pub(crate) fn quick_input_watcher(script: String) {
    // lazy_static! {
    //     static ref SETTINGS: RwLock<Config> = RwLock::new({
    //         let mut settings = Config::default();
    //         settings.merge(File::with_name("cfg.toml")).unwrap();
    //         settings
    //     });
    // }
    control::execute(script);
    // TODO: asyn executing:
    show();
    watch();
}

pub(crate) fn show() {
    println!(
        "* Settings:: \n\x1b[31m{:?}\x1b[0m",
        CFG.read()
            .unwrap()
            .clone()
            .try_deserialize::<Input>()
            .unwrap_or_default()
    );
}

fn watch() {
    let (tx, rx) = channel();

    //LEARN setting up a notify to watch data files and config files may be good
    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher = Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(1)),
    )
    .unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    assert!(watcher
        .watch(Path::new(LOCAL_CFG_PATH), RecursiveMode::NonRecursive)
        .is_ok());

    // This is a simple loop, but you may want to use more complex logic here,
    // for example to handle I/O.
    loop {
        match rx.recv() {
            Ok(Ok(Event {
                kind: notify::event::EventKind::Modify(_),
                ..
            })) => {
                // println!(
                // " * [{}] is modified; refreshing configuration ...",
                // LOCAL_CFG
                // );

                log_func!(" dds input is modified");
                // TODO remove unwraps.
                let input = CFG
                    .write()
                    .unwrap()
                    .refresh()
                    .unwrap()
                    .clone()
                    .try_deserialize::<Input>()
                    .unwrap_or_default();
                if input.valid_input() {
                    data::send_msg(serde_json::to_string_pretty(&input).unwrap_or_default());
                    show();
                } else {
                    println!("{}", "invalid input setting!".on_red());
                }
            }

            Err(e) => println!("watch error: {:?}", e),

            _ => {
                // Ignore event
            }
        }
    }
}

/// very, very, **unsafe**! only support one layor TOML parse
#[test]
pub fn write_to_cfg() {
    let builder = Config::builder()
        .add_source(File::with_name(LOCAL_CFG_PATH))
        .build();
    assert!(builder.is_ok());
    let settings = builder.unwrap();
    assert!(settings.try_deserialize::<Input>().is_ok());
}

#[test]
fn config_demo() {
    let builder = Config::builder()
        .add_source(File::with_name("test.toml"))
        .build();

    if let Ok(settings) = builder {
        assert!(settings.try_deserialize::<Input>().is_ok());
    }
}

#[test]
fn config_global() {
    use std::error::Error;
    use std::sync::RwLock;

    lazy_static! {
        static ref SETTINGS: RwLock<Config> = RwLock::new(Config::default());
    }
    fn try_main() -> Result<(), Box<dyn Error>> {
        let _ = SETTINGS.write()?.set("hostname", "DDS-Controller");
        println!(
            "\thostname => {}",
            SETTINGS.read()?.get::<String>("hostname")?
        );

        Ok(())
    }

    try_main();
}
