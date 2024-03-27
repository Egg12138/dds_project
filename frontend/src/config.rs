#![allow(deprecated)]
use config::{Config, File, FileStoredFormat, Format, Value, ValueKind};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{collections::HashMap, env::set_current_dir, error::Error, sync::RwLock};

const CFG_WATCH_INT: i32 = 2; // in seconds

#[derive(Debug, Deserialize, Serialize)]
pub struct MyConfig {
    pub host_name: String,
    pub default_baud_rate: usize,
    pub default_freq_hz: usize,
}

// input the string of freq
pub(crate) fn freq_valid(freq: String) -> bool {
    true
}

pub(crate) fn vol_valid(v: String) -> bool {
    true
}

pub(crate) fn ph_valid(p: String) -> bool {
    true
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

pub(crate) fn test_watch() {
    use super::*;
    use notify::{Event, EventHandler, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::{mpsc::channel, RwLock};
    use std::time::Duration;
    lazy_static! {
        static ref SETTINGS: RwLock<Config> = RwLock::new({
            let mut settings = Config::default();
            settings.merge(File::with_name("cfg.toml")).unwrap();
            settings
        });
    }

    fn show() {
        println!(
            "* Settings:: \n\x1b[31m{:?}\x1b[0m",
            SETTINGS
                .read()
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
            .watch(Path::new("cfg.toml"), RecursiveMode::NonRecursive)
            .is_ok());

        // This is a simple loop, but you may want to use more complex logic here,
        // for example to handle I/O.
        // TODO refactor as a function:
        // TODO [easy] apply the valid config.
        // TODO [easy] write a .tmp backup to store the correct version.
        loop {
            match rx.recv() {
                Ok(Ok(Event {
                    kind: notify::event::EventKind::Modify(_),
                    ..
                })) => {
                    println!(" * cfg.toml is modified; refreshing configuration ...");
                    // TODO remove unwraps.
                    SETTINGS.write().unwrap().refresh().unwrap();
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
