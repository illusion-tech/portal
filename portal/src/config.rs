use super::*;
// use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};

const HOST_ENV: &str = "CTRL_HOST";
const PORT_ENV: &str = "CTRL_PORT";
const TLS_OFF_ENV: &str = "CTRL_TLS_OFF";

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_CONTROL_HOST: &str = "localhost";
const DEFAULT_CONTROL_PORT: &str = "5000";

const SETTINGS_DIR: &str = ".portal";
const SECRET_KEY_FILE: &str = "key.token";

/// Config
#[derive(Debug, Clone)]
pub struct Config {
    pub client_id: ClientId,
    pub control_url: String,
    pub use_tls: bool,
    pub host: String,
    pub local_host: String,
    pub local_port: u16,
    pub local_addr: SocketAddr,
    pub sub_domain: Option<String>,
    pub secret_key: Option<SecretKey>,
    pub control_tls_off: bool,
    pub dashboard_port: u16,
    pub verbose: bool,
}

impl Config {
    /// Parse the URL to use to connect to the wormhole control server
    pub fn load() -> Result<Config, ()> {
        if CLI.verbose {
            std::env::set_var("RUST_LOG", "portal=debug");
        }

        pretty_env_logger::init();

        let secret_key: Option<String> = None;
        let sub_domain = CLI.sub_domain.clone();

        let local_addr = (CLI.local_host.as_str(), CLI.port)
            .to_socket_addrs()
            .map_err(|_| {
                error!(
                    "Failed to resolve local address: {}:{}",
                    CLI.local_host.as_str(),
                    CLI.port
                )
            })?
            .next()
            .ok_or_else(|| {
                error!(
                    "No IP addresses found for: {}:{}",
                    CLI.local_host.as_str(),
                    CLI.port
                )
            })?;

        // get the host url
        let tls_off = env::var(TLS_OFF_ENV).is_ok();
        let host = env::var(HOST_ENV).unwrap_or(DEFAULT_HOST.to_string());

        let control_host = env::var(HOST_ENV).unwrap_or(DEFAULT_CONTROL_HOST.to_string());

        let port = env::var(PORT_ENV).unwrap_or(DEFAULT_CONTROL_PORT.to_string());

        let scheme = if tls_off { "ws" } else { "wss" };
        let control_url = format!("{}://{}:{}/wormhole", scheme, control_host, port);

        info!("Control Server URL: {}", &control_url);

        Ok(Config {
            client_id: ClientId::generate(),
            local_host: CLI.local_host.clone(),
            use_tls: CLI.use_tls,
            control_url,
            host,
            local_port: CLI.port,
            local_addr,
            sub_domain,
            dashboard_port: CLI.dashboard_port.unwrap_or(0),
            verbose: CLI.verbose,
            secret_key: secret_key.map(SecretKey),
            control_tls_off: tls_off,
        })
    }

    pub fn activation_url(&self, full_hostname: &str) -> String {
        format!(
            "{}://{}",
            if self.control_tls_off {
                "http"
            } else {
                "https"
            },
            full_hostname
        )
    }

    pub fn forward_url(&self) -> String {
        let scheme = if self.use_tls { "https" } else { "http" };
        format!("{}://{}:{}", &scheme, &self.local_host, &self.local_port)
    }
    pub fn ws_forward_url(&self) -> String {
        let scheme = if self.use_tls { "wss" } else { "ws" };
        format!("{}://{}:{}", scheme, &self.local_host, &self.local_port)
    }
}
