use clap::Parser;

use crate::cli::SubCommands;

// macros
// command line input parse
// communicate with esp32 (Rust on esp32)
mod cli;

fn main() {
    println!("Hello, world!");
    let cli = cli::Cli::parse();
    if let Some(name) = cli.name.as_deref() {
        println!("name: {name}");
    }
    if let Some(config_path) = cli.config.as_deref() {
        println!("config: {}", config_path.display());
    }

    match &cli.commands {
        Some(SubCommands::Monitor(m)) => {
            println!(
                "port {:?} at {:?}:{}",
                m.port,
                m.baud_rate,
                m.baud_rate.unwrap().get().unwrap()
            );
        }

        Some(SubCommands::Run(r)) => println!("instructions {}", r.instruction_input),
        _ => println!("IMPOSSIBLe!"),
    }
}

#[cfg(test)]
mod test {
    use crate::cli::Cli;

    use super::*;
    fn repl_enable() {
        const INPUT: &str = "ddsc repl";
        let cli = Cli::from(INPUT);
        if cli.verbose {}
    }
}
