use serde::Deserialize;

use super::*;
use std::{
    error::Error,
    net::{SocketAddr, ToSocketAddrs},
};

const HOST_ENV: &str = "CTRL_HOST";
const PORT_ENV: &str = "CTRL_PORT";
const TLS_OFF_ENV: &str = "CTRL_TLS_OFF";

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_CONTROL_HOST: &str = "localhost";
const DEFAULT_CONTROL_PORT: &str = "5000";

const SETTINGS_DIR: &str = ".portal";
const SECRET_KEY_FILE: &str = "key.token";

#[derive(Deserialize, Debug)]
struct InternalConfig {
    sub_domain: Option<String>,
    portal_host: Option<String>,
    portal_port: Option<u16>,
    portal_tls: Option<bool>,
    local_host: Option<String>,
    local_port: Option<u16>,
    local_tls: Option<bool>,
    dashboard_port: Option<u16>,
    verbose: Option<bool>,
}

/// Config
#[derive(Debug, Clone)]
pub struct Config {
    pub client_id: ClientId,
    pub portal_host: String,
    pub portal_port: u16,
    pub portal_tls: bool,
    pub local_tls: bool,
    pub local_host: String,
    pub local_port: u16,
    pub local_addr: SocketAddr,
    pub sub_domain: Option<String>,
    pub secret_key: Option<SecretKey>,
    pub dashboard_port: u16,
    pub verbose: bool,
}

impl From<&mut InternalConfig> for Config {
    fn from(config: &mut InternalConfig) -> Self {
        let local_host = config
            .local_host
            .clone()
            .unwrap_or(DEFAULT_HOST.to_string());
        let local_port = config.local_port.unwrap_or(8000);
        let local_addr = (local_host.as_str(), local_port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        let local_tls = config.local_tls.unwrap_or(false);

        let portal_tls = config.portal_tls.unwrap_or(false);
        let portal_schema = if portal_tls { "wss" } else { "ws" };
        let portal_host = config
            .portal_host
            .take()
            .unwrap_or(DEFAULT_CONTROL_HOST.to_string());
        let portal_port = config.portal_port.unwrap_or(5000);
        let portal_url = format!(
            "{}://{}:{}/wormhole",
            portal_schema, portal_host, portal_port
        );
        let secret_key = None.map(SecretKey);
        let dashboard_port = config.dashboard_port.unwrap_or(0);
        let verbose = config.verbose.unwrap_or(false);

        Config {
            client_id: ClientId::generate(),
            sub_domain: config.sub_domain.take(),
            local_host,
            local_port,
            local_addr,
            local_tls,
            portal_host,
            portal_port,
            portal_tls,
            secret_key,
            dashboard_port,
            verbose,
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        let config = std::fs::read_to_string(path)?;
        let mut config: InternalConfig = toml::from_str(&config)?;
        std::env::set_var("RUST_LOG", "portal=debug");
        Ok(Config::from(&mut config))
    }

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
        let portal_host = env::var(HOST_ENV).unwrap_or(DEFAULT_CONTROL_HOST.to_string());
        let portal_port = env::var(PORT_ENV).unwrap_or(DEFAULT_CONTROL_PORT.to_string());

        info!("Control Server URL: {}", &portal_host);

        Ok(Config {
            client_id: ClientId::generate(),
            portal_host,
            portal_port: portal_port.parse().unwrap(),
            local_host: CLI.local_host.clone(),
            local_port: CLI.port,
            local_tls: CLI.use_tls,
            local_addr,
            sub_domain,
            dashboard_port: CLI.dashboard_port.unwrap_or(0),
            verbose: CLI.verbose,
            secret_key: secret_key.map(SecretKey),
            portal_tls: !tls_off,
        })
    }

    pub fn activation_url(&self, full_hostname: &str) -> String {
        format!(
            "{}://{}",
            if self.portal_tls { "http" } else { "https" },
            full_hostname
        )
    }

    pub fn forward_url(&self) -> String {
        let scheme = if self.local_tls { "https" } else { "http" };
        format!("{}://{}:{}", &scheme, &self.local_host, &self.local_port)
    }
    pub fn ws_forward_url(&self) -> String {
        let scheme = if self.local_tls { "wss" } else { "ws" };
        format!("{}://{}:{}", scheme, &self.local_host, &self.local_port)
    }

    pub fn portal_url(&self) -> String {
        format!(
            "{}://{}:{}/wormhole",
            self.portal_schema(), self.portal_host, self.portal_port
        )
    }

    pub fn portal_schema(&self) -> &str {
        if self.portal_tls {
            "wss"
        } else {
            "ws"
        }
    }
}
