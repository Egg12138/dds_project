use clap::{crate_version, Args, Parser, Subcommand, ValueEnum};
// use serial_core::BaudRate;
use std::{fmt::Display, path::PathBuf};

// GOAL: render all options in colors
/// Cli
#[derive(Parser)]
#[command(name = "DDS Controller Frontend")]
#[command(version = crate_version!())]
#[command(propagate_version = false)]
#[command(long_version = "
	MCU: CORE-ESP32C3,
	DDS module: ad9910(for example)
	binary: ddsc
	instruction runner: v0.0.1
	repl: v0.0.1
	")]
#[command(about, long_about = "Command Line DDS-Controller")]
#[command(bin_name = "ddsc")]
#[command(propagate_version = true)]
// #[command(next_line_help = true)]
// #[command(debug_assert)]
pub(super) struct Cli {
    #[arg(long, help = "set the host name")]
    pub(super) name: Option<String>,

    #[arg(long, help = "initialize the system and do primary check")]
    pub(super) init: bool,
    #[arg(
        value_parser,
        short = 'm',
        long,
        help = " how Host to ESP32,iot | wifi | ble | wired(default)",
        default_value = "wired"
    )]
    pub(super) mode: Option<CommunicationMethod>,

    #[arg(short, long, value_name = "FILE")]
    pub(super) config: Option<PathBuf>,

    /// count: increment a `u8` counter
    /// default will be 0 is `default_value` is not set
    #[cfg(test)]
    #[arg(short, long, action = clap::ArgAction::Count, default_value = "1")]
    pub(crate) test: u8,

    #[deprecated(since = "0.1.1", note = "the remote SSID passing is unsafe")]
    pub(super) spec_ssid: Option<String>,
    #[deprecated(since = "0.1.1", note = "the remote SSID passing is unsafe")]
    pub(super) spec_pwd: Option<String>,

    /// example:
    /// $ ddsc poweroff=mcu:3000
    // LEARN how to impl git like git local:remote option
    #[arg(long, value_name = "POWEROFF:wait")]
    pub(super) poweroff: Vec<String>,

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
    pub(super) commands: Option<SubCommands>,
}

#[derive(Subcommand)]
pub(super) enum SubCommands {
    Run(RunnerArgs),
    Monitor(MonitorArgs),
}

#[derive(Args)]
pub(crate) struct RunnerArgs {
    #[arg(
        short,
        long = "input",
        help = "the input the instructions, files OR string"
    )]
    //TODO: support input <PathBuf/String>
    pub(super) instruction_input: String,
}

#[derive(Args)]
pub(crate) struct MonitorArgs {
    #[arg(short, long, help = "e.g. COM6 (on windows), /dev/tty2 (on linux)")]
    pub port: String,
    #[arg(short, long, default_value = "Baud115200")]
    pub baud_rate: Option<BaudRate>,
}

#[derive(Parser)]
pub(crate) enum TODOS {}

/// only IoT options is using the remote IoT
#[derive(ValueEnum, Clone, Copy)]
pub(crate) enum CommunicationMethod {
    /// must via WLAN
    IoT,
    Wifi,
    Ble,
    Wired,
}

//--------------------

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

impl BaudRate {
    pub(crate) fn get(&self) -> Result<usize, <usize as std::str::FromStr>::Err> {
        let variant_field = format!("{:?}", self);
        variant_field[4..].parse::<usize>()
    }
}
