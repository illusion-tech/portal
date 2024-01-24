use clap::{Parser, Subcommand};

/// The CLI options for the portal
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// A level of verbosity, and can be used multiple times
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Sets an API authentication key to use for this portal
    #[arg(short, long)]
    pub key: Option<String>,

    /// Specify a sub-domain for this portal
    #[arg(short, long)]
    pub sub_domain: Option<String>,

    /// Sets the HOST (i.e. localhost) to forward incoming portal traffic to
    #[arg(long = "host", default_value = "localhost")]
    pub local_host: String,

    /// Sets the protocol for local forwarding (i.e. https://localhost) to forward incoming portal traffic to
    #[arg(long = "use-tls", short = 't')]
    pub use_tls: bool,

    /// Sets the port to forward incoming portal traffic to on the target host
    #[arg(short, long, default_value = "8000")]
    pub port: u16,

    /// Sets the address of the local introspection dashboard
    #[arg(long = "dashboard-port")]
    pub dashboard_port: Option<u16>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Store the API Authentication key
    SetAuth {
        /// Sets an API authentication key on disk for future use
        #[arg(short, long)]
        key: String,
    },
}
