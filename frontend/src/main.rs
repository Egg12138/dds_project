#![allow(
    deprecated,
    non_upper_case_globals,
    non_snake_case,
    clippy::upper_case_acronyms
)]
#![feature(ip_bits, const_refs_to_static, div_duration)]

use clap::Parser;

use crate::cli::Cmds;
use colored::Colorize;

// macros
// command line input parse
// communicate with esp32 (Rust on esp32)
mod cli;
mod config;
mod control;
mod data;
mod ddserror;
mod nets;
mod rawtests;
mod register_controller;

use cli::Cli;

fn main() {
    let args = Cli::parse();

    match args.commands {
        Cmds::Repl { interactive } => {
            if interactive.unwrap() {
                control::repl();
            } else {
                println!("CLI mode");
            }
        }

        Cmds::PowerOff { wait } => control::poweroff(wait),

        Cmds::Run(runner) => control::run(runner),

        Cmds::Monitor(_) => control::monitor(),
        _ => {}
    }

    log_func!();
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::Subcommand;

    const CMDS: [&'static str; 8] = [
        "init", "config", "repl", "power", "repl", "run", "monitor", "data",
    ];

    #[test]
    fn all_subcmds_ready() {
        let _ = CMDS.into_iter().map(|cmd| {
            assert!(Cmds::has_subcommand(&cmd));
        });
    }

    #[test]
    fn repl_enable() {
        const INPUT: [&'static str; 2] = ["ddsc", "repl"];
        let cli = Cli::parse_from(INPUT);
    }

    #[test]
    fn verifys() {
        use super::config;
    }
}
