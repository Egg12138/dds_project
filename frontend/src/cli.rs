// TODO: REMOVE unnacassary Debug attribute!
use clap::{crate_version, Args, Parser, Subcommand, ValueEnum};
// use serial_core::BaudRate;
use std::{arch::x86_64::_CMP_ORD_Q, fmt::Display, path::PathBuf};

#[cfg(target_os = "windows")]
const DEFAULT_NAME: &'static str = "MagicBook Windows";
#[cfg(target_os = "linux")]
const DEFAULT_NAME: &'static str = "MagicBook Linux";

const LONG_VER: &'static str = "
	MCU: CORE-ESP32C3,
	DDS module: ad9910(for example)
	binary: ddsc
	instruction runner: v0.0.1
	repl: v0.0.1
";

// GOAL: render all options in colors
/// Cli
#[derive(Debug)] // TODO: remove the attribute
#[derive(Parser)]
#[command(name = "DDS Controller Frontend")]
#[command(version = crate_version!())]
#[command(propagate_version = false)]
#[command(long_version = LONG_VER)]
#[command(about, long_about = "Command Line DDS-Controller")]
#[command(bin_name = "ddsc")]
#[command(propagate_version = true)]
// #[command(next_line_help = true)]
// #[command(debug_assert)]
pub(crate) struct Cli {
    #[arg(long, help = "set the host name", default_value = DEFAULT_NAME)]
    pub(super) name: Option<String>,

    #[arg(long, help = "initialize the system and do primary check")]
    pub(crate) init: bool,
    #[arg(
        short = 'm',
        long,
        help = " how Host to ESP32,iot | wifi | ble | wired(default)",
        default_value = "wired"
    )]
    pub(crate) mode: Option<CommunicationMethod>,

    #[arg(short, long, value_name = "FILE")]
    pub(crate) config: Option<PathBuf>,

    /// count: increment a `u8` counter
    /// default will be 0 is `default_value` is not set
    #[cfg(test)]
    #[arg(short, long, action = clap::ArgAction::Count, default_value = "1")]
    pub(super) test: u8,

    #[deprecated(since = "0.1.1", note = "the remote SSID passing is unsafe")]
    pub(super) spec_ssid: Option<String>,
    #[deprecated(since = "0.1.1", note = "the remote SSID passing is unsafe")]
    pub(super) spec_pwd: Option<String>,

    /// example:
    /// $ ddsc poweroff=mcu:3000
    // LEARN how to impl git like git local:remote option
    #[arg(long, value_name = "POWEROFF:wait")]
    pub(crate) poweroff: Option<String>,

    /// enable the REPL mode (if repl is sepecified, other options will be ignored)
    #[arg(
        long = "repl",
        help = "enter interactive mode",
        default_value = "false"
    )]
    interactive: bool,

    #[arg(short, long)]
    pub(super) verbose: bool,

    #[command(subcommand)]
    pub(crate) commands: Option<SubCommands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum SubCommands {
    Run(RunnerArgs),
    Monitor(MonitorArgs),
}

#[derive(Debug)]
#[derive(Args)]
pub(crate) struct RunnerArgs {
    #[arg(
        short,
		value_name = "INSTRUCTIONS",
        long = "input",
        help = "the input the instructions, files OR string",
		default_value = "{}"
    )]
    //TODO: support input <PathBuf/String>
    pub(super) instruction_input: Option<String>,
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
#[derive(ValueEnum, Clone, Copy, Debug)]
pub(crate) enum CommunicationMethod {
    /// must via WLAN
    IoT,
    Wifi,
    Ble,
    Wired,
}

impl Display for CommunicationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = match self {
            Self::IoT => "IoT",
            Self::Wifi => "WLAN,non-IoT",
            Self::Ble => "Bluetooth",
            Self::Wired => "GPIO Wired",
        };
        write!(f, "{}", mode)
    }
}

//TODO remove publice disgs of fileds
pub trait FetchInfo {
	fn host_name(&self) -> &'static str;
	fn mode(&self) -> CommunicationMethod;
	/// return port
	fn monitor(&self) -> Option<(String, usize)>;
	fn instruction(&self) -> Option<RunnerArgs>;
	fn version(&self) -> &'static str;
}

impl FetchInfo for Cli {
	fn host_name(&self) -> &'static str {
		DEFAULT_NAME	
	}	
	fn mode(&self) -> CommunicationMethod {
		self.mode.unwrap()
	}
	fn monitor(&self) -> Option<(String, usize)> {
		// IMPL
		todo!()
	}
	fn instruction(&self) -> Option<RunnerArgs> {
		// IMPL
		todo!()
	}
	 fn version(&self) -> &'static str {
		LONG_VER
	}
}
























//NOTICE remove deprecated parts --------------------

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_field = format!("{:?}", self);
        write!(f, "{:?}", &variant_field[4..])
    }
}

#[deprecated(since = "0.1.1", note = "it's better to directly parse number as baud rate!")]
impl BaudRate {
    /// actually, the return is undoubely valid . I still return `Result`
    pub(crate) fn get(&self) -> Result<usize, <usize as std::str::FromStr>::Err> {
        let variant_field = format!("{:?}", self);
        variant_field[4..].parse::<usize>()
    }
}
