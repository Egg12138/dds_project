use clap::{Args, Parser};

#[derive(Parser)]
#[command(name = "DDS Controller Frontend")]
#[command(bin_name = "ddsc")]
pub(crate) enum Cli {
    Method(CommunicationMethod),
}

#[derive(Args)]
#[command(version, about, long_about = None)]
pub(crate) struct CommunicationMethod {}
