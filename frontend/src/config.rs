//! the config.rs checks the validation of input arguments

#![allow(deprecated)]

use crate::cli::CommunicationMethod;
use crate::control;
use crate::data;
use crate::ddserror::DDSError;
use crate::log_func;
use colored::Colorize;
use config::{Config, ConfigError, Environment, File};
use lazy_static::lazy_static;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[allow(unused_imports)]
use std::fmt::write;
#[allow(unused_imports)]
use std::future::Future;
use std::net::Ipv4Addr;
use std::path::Path;
//use std::str::FromStr;
use std::sync::{mpsc::channel, RwLock};
use std::time::Duration;

const CFG_WATCH_INT: u64 = 2; // in seconds
/// the `dds input` local configuration TOML file in the {workspace},
const LOCAL_CFG: &str = "./config/cfg.toml";
/// mcu settings
const LOCAL_MCU_CFG: &str = "./config/mcucfg.toml";
const DEFAULT_MCU_CFG: &str = "/.config/default_mcgcfg.toml";
const ENV_PREFIX: &str = "dds";

lazy_static! {
    static ref CFG: RwLock<Config> = RwLock::new({
        let mut configuration = Config::default();
        configuration.merge(File::with_name(LOCAL_CFG)).unwrap();
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
    mode: CommunicationMethod,
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
            .add_source(File::with_name(LOCAL_MCU_CFG))
            .add_source(File::with_name(DEFAULT_MCU_CFG).required(false))
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
    pub(crate) fn mode(&self) -> CommunicationMethod {
        self.connection.mode
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicationTypes {
    SetInput,
    PowerOff,
    Scan,
    Display,
    /// report the status of MCU and DDS
    Report,
}

impl std::fmt::Display for IndicationTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetInput => write!(f, "setinput"),
            Self::PowerOff => write!(f, "poweroff"),
            Self::Scan => write!(f, "scan"),
            Self::Display => write!(f, "display"),
            Self::Report => write!(f, "report"),
        }
    }
}

/// a standard collection of data to be sent.
/// but `quick_Watcher` is a faster way to sending messages.
#[repr(C)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub indication: IndicationTypes,
    pub freq_hz: f64,
    pub vol_mv: f32,
    pub ph_oft: i8,
    // pub collect: bool,
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

        eprintln!("builtin inputs are used");

        Input {
            freq_hz: 0_f64,
            vol_mv: 0_f32,
            ph_oft: 0,
            indication: IndicationTypes::SetInput,
        }
    }
}

impl From<Config> for Input {
    fn from(value: Config) -> Self {
        value.try_deserialize().unwrap_or_default()
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

    pub(crate) fn new(path: &str) -> Self {
        if let Ok(builder) = Config::builder().add_source(File::with_name(path)).build() {
            builder.try_deserialize().unwrap_or_default()
        } else {
            eprintln!("failed to build config from the {}", path);
            Input::default()
        }
    }

    fn handle(&mut self) {
        let cfgmap = CFG.read().unwrap().clone();
        let input = Input::from(cfgmap);
        self.freq_hz = input.freq_hz;
        self.vol_mv = input.vol_mv;
        self.ph_oft = input.ph_oft;
        self.indication = input.indication;

        println!(
            "* Input:: 
        \n\x1b[31m {:?}\x1b[0m",
            self
        );
        let encoded = serde_json::to_string(&self).unwrap_or_default();
        data::send_msg(encoded);
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
            .watch(Path::new(LOCAL_CFG), RecursiveMode::NonRecursive,)
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
    data::wait4response_show();
    watch();
}

pub(crate) fn show() {
    println!(
        "* Settings:: \n\x1b[31m{:?}\x1b[0m",
        CFG.read()
            .unwrap()
            .clone()
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
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
        .watch(Path::new(LOCAL_CFG), RecursiveMode::NonRecursive)
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
        .add_source(File::with_name("cfg.toml"))
        .build();
    if let Ok(settings) = builder {
        println!(
            "{:#?}",
            settings
                .try_deserialize::<HashMap<String, String>>()
                .unwrap() // .unwrap_or(HashMap::default())
        );
    } else {
        panic!("Failed to build cfg.toml!");
    }
}

#[test]
fn config_demo() {
    use super::*;
    use std::collections::HashMap;
    let builder = Config::builder()
        .add_source(File::with_name("test.toml"))
        .build();

    if let Ok(settings) = builder {
        println!(
            "{:?}",
            settings
                .try_deserialize::<HashMap<String, String>>()
                .unwrap()
        );
    }
}

#[test]
fn config_global() {
    use super::*;
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
