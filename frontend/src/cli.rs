// TODO: REMOVE unnacassary Debug attribute!
use clap::{arg, crate_version, 
    Args, Parser, 
    Subcommand, ValueEnum, 
    ArgAction};
use serde::Deserialize;
// use serial_core::BaudRate;
use std::{fmt::Display, path::PathBuf};

#[cfg(target_family = "windows")]
#[allow(dead_code)]
pub(crate) const DEFAULT_NAME: &str = "MagicBook Windows";
#[allow(dead_code)]
#[cfg(target_family = "linux")]
pub(crate) const DEFAULT_NAME: &str = "MagicBook Unix";

const MODE_HELP: &str = " 
    how Host to ESP32: iot | wired | ble | wife(default)
    it's hightly recommanded to directly use the screen to controll DDS 
    if the screen is touchable.
    Only if you'd like to collect data for advanced operations or the screen 
    is not touchable, use the front end.
    IoT mode is better then Wifi/ble mode, excepting you are considering about 
    somehow `privacy`?
    ";

const LONG_VER: &str = "
	MCU: CORE-ESP32C3,
	DDS module: ad9910(for example)
	binary: ddsc
	instruction runner: v0.0.1
	repl: v0.0.1
";

// GOAL: render all options in colors
/// Cli
#[derive(Parser)]
#[command(name = "DDS Controller Frontend")]
#[command(version = crate_version!())]
#[command(long_version = LONG_VER)]
#[command(propagate_version = false)]
#[command(about, long_about = "Command Line DDS-Controller")]
#[command(bin_name = "ddsc")]
#[command(propagate_version = true)]
// #[command(next_line_help = true)]
// #[command(debug_assert)]
pub(crate) struct Cli {


    #[command(subcommand)]
    pub(crate) commands: Cmds,
}

#[derive(Subcommand)]
pub(crate) enum Cmds {





    #[deprecated]
    #[command(arg_required_else_help = true)]
    Config {
        #[deprecated(since = "0.1.1", note = "the remote SSID passing is unsafe")]
        #[arg(long, hide = true)]
        spec_ssid: Option<String>,
        #[deprecated(since = "0.1.1", note = "the remote SSID passing is unsafe")]
        #[arg(long, hide = true)]
        spec_pwd: Option<String>,
            // TODO wavetable path parse
        // #[arg(long, hide = true)]
        // new_wavetable: Option<PathBuf>,
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },


    PowerOff{
    /// example:
    /// $ ddsc power=mcu:3000
    // LEARN how to impl git like git local:remote option
        #[arg(default_value = "0")]
        wait: Option<u64>
    },

    PowerOn,

    

    Repl {
        /// enable the REPL mode (if repl is sepecified, other options will be ignored)
        /// `ddsc interactive`
        #[arg(
            long = "interactive",
            action = ArgAction::SetFalse ,
            help = "enter interactive mode",
            default_value = "true", 
        )]
        interactive: Option<bool>,
    },

    #[command(arg_required_else_help = true)]
    Run(RunnerArgs),
    #[command(arg_required_else_help = true)]
    Monitor(MonitorArgs),
    #[command(arg_required_else_help = true)]
    Data(DataArgs),
}

impl Display for Cmds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", std::any::TypeId::of::<Self>())
    }
}

#[derive(Args)]
pub(crate) struct RunnerArgs {
    #[arg(
		value_name = "INSTRUCTIONS",
        help = "the input the instructions, files OR string",
    )]
    //TODO: support input <PathBuf/String>
    pub(super) instruction_input: String,
    // pub(super) input: Option<PathBuf>,
        #[arg(
        short, long, 
        help = MODE_HELP,
        )]
        pub(super) mode: CommunicationMethod,


}

#[derive(Debug)]
#[derive(Args)]
pub(crate) struct MonitorArgs {
    #[arg(short, long, help = "e.g. COM6 (on windows), /dev/tty2 (on linux)")]
    pub port: String,
    #[arg(short, 
		long, 
		value_parser = baudrate_range,
		default_value = "115200")]
    pub baud_rate: Option<usize>,
}

#[derive(Debug)]
#[derive(Args)]
pub(crate) struct DataArgs {
    #[arg(short, long, help = "")]
    pub format: Option<String>,
}

const BAUD_RATE_RANGE: [usize; 18] =  [
	110,
    300,
    600,
    1200,
    2400,
    4800,
    9600,
    19200,
    38400,
    57600,
    115200,
    230400,
    460800,
    512000,
    921600,
    1000000,
    1152000,
    1500000,
];

fn baudrate_range(brstr: &str) -> Result<usize, String> {
	let br: usize = brstr
    .parse()
    .map_err(| _ | format!("{brstr} isn't a valid baud rate" ))?;
	if BAUD_RATE_RANGE.contains(&br) {
		Ok(br)
	} else {
		Err(format!("invalid baud rate! {}", br))
	}
}

/// only IoT options is using the remote IoT
#[derive(ValueEnum, Clone, Copy, Debug, Deserialize)]
pub(crate) enum CommunicationMethod {
    /// must via WLAN
    Iot,
    Wifi,
    Ble,
    Wired,
}

impl Display for CommunicationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = match self {
            Self::Iot => "IoT",
            Self::Wifi => "WLAN,non-IoT",
            Self::Ble => "Bluetooth",
            Self::Wired => "GPIO Wired",
        };
        write!(f, "{}", mode)
    }
}




















//NOTICE remove deprecated parts --------------------

#[allow(unused, deprecated)]
#[deprecated(since = "0.1.1", note = "it's better to directly parse number as baud rate!")]
#[derive(Debug, ValueEnum, Clone, Copy)]
pub(crate) enum BaudRate {
    Baud110,
    Baud300,
    Baud600,
    Baud1200,
    Baud2400,
    Baud4800,
    Baud9600,
    Baud19200,
    Baud38400,
    Baud57600,
    Baud115200,
    Baud230400,
    Baud460800,
    Baud512000,
    Baud921600,
    Baud1000000,
    Baud1152000,
    Baud1500000,
}

impl Display for BaudRate {
#[allow(unused, deprecated)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_field = format!("{:?}", self);
        write!(f, "{:?}", &variant_field[4..])
    }
}

#[deprecated(since = "0.1.1", note = "it's better to directly parse number as baud rate!")]
impl BaudRate {
    /// actually, the return is undoubely valid . I still return `Result`
#[allow(unused, deprecated)]
    pub(crate) fn get(&self) -> Result<usize, <usize as std::str::FromStr>::Err> {
        let variant_field = format!("{:?}", self);
        variant_field[4..].parse::<usize>()
    }
}
