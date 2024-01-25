use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Use a toml file for configuration.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}
