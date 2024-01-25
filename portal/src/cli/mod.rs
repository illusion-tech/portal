use core::time::Duration;
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::{Config, FIRST_RUN};
use clap::{Parser, Subcommand};
use cli_table::format::Padding;
use cli_table::{format::Justify, print_stderr, Cell, Table};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

/// The CLI options for the portal
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// A level of verbosity, and can be used multiple times
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Use a toml file for configuration.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

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

pub struct CliInterface {
    spinner: ProgressBar,
    config: Config,
    introspect: SocketAddr,
}
impl CliInterface {
    pub fn start(config: Config, introspect: SocketAddr) -> Self {
        let spinner = new_spinner("Opening remote tunnel...");
        Self {
            spinner,
            config,
            introspect,
        }
    }

    fn get_sub_domain_notice(&self, sub_domain: &str) -> Option<String> {
        if self.config.sub_domain.is_some()
            && (self.config.sub_domain.as_deref() != Some(sub_domain))
        {
            if self.config.secret_key.is_some() {
                Some(format!("{}",
                          "To use custom sub-domains feature, please upgrade your billing plan at https://dashboard.tunnelto.dev.".yellow()))
            } else {
                Some(format!("{}",
                          "To access the sub-domain feature, get your authentication key at https://dashboard.tunnelto.dev.".yellow()))
            }
        } else {
            None
        }
    }

    pub async fn did_connect(&self, sub_domain: &str, full_hostname: &str) {
        self.spinner
            .finish_with_message("Success! Remote tunnel is now open.\n".green().to_string());

        if !*FIRST_RUN.lock().await {
            return;
        }

        let public_url = self.config.activation_url(full_hostname).bold().green();
        let forward_url = self.config.forward_url();
        let inspect = format!("http://localhost:{}", self.introspect.port());

        let table = vec![
            vec![
                "Public tunnel URL".green().cell(),
                public_url
                    .green()
                    .cell()
                    .padding(Padding::builder().left(4).right(4).build())
                    .justify(Justify::Left),
            ],
            vec![
                "Local inspect dashboard".magenta().cell(),
                inspect
                    .magenta()
                    .cell()
                    .padding(Padding::builder().left(4).build())
                    .justify(Justify::Left),
            ],
            vec![
                "Forwarding traffic to".cell(),
                forward_url
                    .cell()
                    .padding(Padding::builder().left(4).build())
                    .justify(Justify::Left),
            ],
        ];

        let table = table.table();
        print_stderr(table).expect("failed to generate starting terminal user interface");

        if let Some(notice) = self.get_sub_domain_notice(sub_domain) {
            eprintln!("\n{}: {}\n", ">>> Notice".yellow(), notice);
        }
    }
}

fn new_spinner(message: &'static str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(150));
    pb.set_style(
        ProgressStyle::default_spinner()
            // .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"])
            .tick_strings(&["🌎", "🌍", "🌏"])
            .template("{spinner:.blue} {msg}")
            .expect("Failed to parse template"),
    );
    pb.set_message(message);
    pb
}
