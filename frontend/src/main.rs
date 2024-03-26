use clap::Parser;

use crate::cli::SubCommands;

// macros
// command line input parse
// communicate with esp32 (Rust on esp32)
mod cli;
mod utils;

fn main() {
    let cli = cli::Cli::parse();
    println!("{:#?}", cli);
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
