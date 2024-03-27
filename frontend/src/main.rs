use clap::Parser;

use crate::cli::Cmds;

// macros
// command line input parse
// communicate with esp32 (Rust on esp32)
mod cli;
mod config;
mod control;

use cli::Cli;

fn main() {
    let args = Cli::parse();
    println!("{:#?}", args);

    match args.commands {
        Cmds::Repl { interactive } => {
            if interactive.unwrap() {
                control::repl();
            } else {
                println!("CLI mode");
            }
        }

        Cmds::Init { mode } => control::init_system(mode),
        Cmds::Run(runner) => {
            config::test_watch();
        }
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::Cli;
    use clap::{Command, Subcommand};

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
