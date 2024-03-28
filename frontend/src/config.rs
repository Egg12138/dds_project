#![allow(deprecated)]
use crate::control;
use config::{Config, ConfigError, File, FileFormat, FileStoredFormat, Format, Value, ValueKind};
use lazy_static::lazy_static;
use notify::{Event, EventHandler, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::Path;
use std::sync::{mpsc::channel, RwLock};
use std::time::Duration;
use std::{collections::HashMap, error::Error};

const CFG_WATCH_INT: u64 = 2; // in seconds
/// the local configuration TOML file in the {workspace},
/// config.rs will handle it first.
const LOCAL_CFG: &str = "cfg.toml";

lazy_static! {
    static ref CFG: RwLock<Config> = RwLock::new({
        let mut configuration = Config::default();
        configuration.merge(File::with_name(LOCAL_CFG)).unwrap();
        configuration
    });
}

#[repr(C)]
#[derive(Debug, Deserialize, Serialize)]
pub struct DDSConfig {
    pub host_name: String,
    pub maxfreq_hz: f64,
    pub maxvol_mv: f64,
}

impl DDSConfig {
    // TODO add_source(Environment::with_prefix("ddscfg"))
    // let host_name = crate::cli::DEFAULT_NAME.to_string();
}

#[repr(C)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub freq_hz: f64,
    pub vol_mv: f32,
    pub ph_oft: i8,
    pub collect: bool,
}

impl Default for Input {
    /// read from cfg.toml, or else return the default inputs.
    fn default() -> Self {
        let path = Path::new("cfg.toml");
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
            collect: false,
        }
    }
}

impl From<Config> for Input {
    fn from(value: Config) -> Self {
        value.try_deserialize().unwrap_or_default()
    }
}

impl Input {
    #[deprecated]
    fn _syslevel_path() -> &'static str {
        #[cfg(target_os = "windows")]
        const SysPATH: &str = "%APPDATA%\\dds_controller\\cfg.toml";

        #[cfg(target_os = "linux")]
        const SysPATH: &str = "~/.config/dds_controller//cfg.toml";

        SysPATH
    }

    pub(crate) fn new(path: &str) -> Self {
        if let Ok(builder) = Config::builder()
            .add_source(File::with_name(path.clone()))
            .build()
        {
            builder.try_deserialize().unwrap_or_default()
        } else {
            eprintln!("failed to build config from the {}", path);
            Input::default()
        }
    }

    fn handle(&mut self) {
        let cfgmap = CFG.read().unwrap().clone();
        // IMPL hostname, freq, vol, offset, parser....
        // IMPL invalid?
        // IMPL println!
        // IMPL set to dds(communication)
        // IMPL 然后就是进入communcation模块！
        if let input = Input::from(cfgmap) {
            self.freq_hz = input.freq_hz;
            self.vol_mv = input.vol_mv;
            self.ph_oft = input.ph_oft;
            self.collect = input.collect;
        }

        println!(
            "* Input:: 
        \n\x1b[31m {:?}\x1b[0m",
            self
        );
        let encoded = serde_json::to_string(&self).unwrap_or_default();
        control::send_msg(encoded);
    }

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
                Err(e) => eprintln!("error: {:?}", e),
                _ => {}
            }
        }
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

    pub(crate) fn valid_input(&self, cfg: &Input) -> bool {
        self.freq_valid() && self.vol_valid() && self.ph_valid()
    }
}

pub(crate) fn quick_input_watcher() {
    // lazy_static! {
    //     static ref SETTINGS: RwLock<Config> = RwLock::new({
    //         let mut settings = Config::default();
    //         settings.merge(File::with_name("cfg.toml")).unwrap();
    //         settings
    //     });
    // }

    fn show() {
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
                    println!(
                        " * [{}] is modified; refreshing configuration ...",
                        LOCAL_CFG
                    );
                    // TODO remove unwraps.
                    let input = CFG
                        .write()
                        .unwrap()
                        .refresh()
                        .unwrap()
                        .clone()
                        .try_deserialize::<Input>()
                        .unwrap_or_default();
                    control::send_msg(serde_json::to_string_pretty(&input).unwrap_or_default());
                    show();
                }

                Err(e) => println!("watch error: {:?}", e),

                _ => {
                    // Ignore event
                }
            }
        }
    }

    show();
    watch();
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
