// macros
// command line input parse
// communicate with esp32 (Rust on esp32)
use clap::Arg;
mod cli;

fn main() {
    println!("Hello, world!");
    let cli::Cli::method(args) = cli::Cli;
    if let Some(method) = args.method {
        println!("{:?}", method);
    }
}
