use crate::auth::SigKey;

use std::error::Error;
use std::net::IpAddr;
use std::str::FromStr;

use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct InternalConfig {
    /// What hosts do we allow tunnels on:
    /// i.e:    baz.com => *.baz.com
    ///         foo.bar => *.foo.bar
    allowed_hosts: Option<Vec<String>>,

    /// What sub-domains do we always block:
    /// i.e:    dashboard.tunnelto.dev
    blocked_sub_domains: Option<Vec<String>>,

    /// port for remote streams (end users)
    remote_port: Option<u16>,

    /// port for the control server
    control_port: Option<u16>,

    /// internal port for instance-to-instance gossip communications
    internal_network_port: Option<u16>,

    /// our signature key path
    master_sig_key: Option<String>,

    /// Instance DNS discovery domain for gossip protocol
    gossip_dns_host: Option<String>,

    /// Observability API key
    honeycomb_api_key: Option<String>,

    /// The identifier for this instance of the server
    instance_id: Option<String>,

    /// Blocked IP addresses
    blocked_ips: Option<Vec<IpAddr>>,

    /// The host on which we create tunnels on
    portal_host: Option<String>,
}

/// Global service configuration
#[derive(Debug)]
pub struct Config {
    /// What hosts do we allow tunnels on:
    /// i.e:    baz.com => *.baz.com
    ///         foo.bar => *.foo.bar
    pub allowed_hosts: Vec<String>,

    /// What sub-domains do we always block:
    /// i.e:    dashboard.tunnelto.dev
    pub blocked_sub_domains: Vec<String>,

    /// port for remote streams (end users)
    pub remote_port: u16,

    /// port for the control server
    pub control_port: u16,

    /// internal port for instance-to-instance gossip coms
    pub internal_network_port: u16,

    /// our signature key
    pub master_sig_key: SigKey,

    /// Instance DNS discovery domain for gossip protocol
    pub gossip_dns_host: Option<String>,

    /// Observability API key
    pub honeycomb_api_key: Option<String>,

    /// The identifier for this instance of the server
    pub instance_id: String,

    /// Blocked IP addresses
    pub blocked_ips: Vec<IpAddr>,

    /// The host on which we create tunnels on
    pub portal_host: String,
}

impl From<InternalConfig> for Config {
    fn from(config: InternalConfig) -> Self {
        let allowed_hosts = config.allowed_hosts.unwrap_or_default();
        let blocked_sub_domains = config.blocked_sub_domains.unwrap_or_default();
        let remote_port = config.remote_port.unwrap_or(8080);
        let control_port = config.control_port.unwrap_or(5000);
        let internal_network_port = config.internal_network_port.unwrap_or(6000);
        let master_sig_key = config
            .master_sig_key
            .map(|key| {
                SigKey::from_hex(&key).expect("invalid master key: not hex or length incorrect")
            })
            .unwrap_or_else(SigKey::generate);
        let gossip_dns_host = config.gossip_dns_host;
        let honeycomb_api_key = config.honeycomb_api_key;
        let instance_id = config
            .instance_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let blocked_ips = config.blocked_ips.unwrap_or_default();
        let portal_host = config
            .portal_host
            .unwrap_or_else(|| "tunnelto.dev".to_string());

        Config {
            allowed_hosts,
            blocked_sub_domains,
            remote_port,
            control_port,
            internal_network_port,
            master_sig_key,
            gossip_dns_host,
            honeycomb_api_key,
            instance_id,
            blocked_ips,
            portal_host,
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        info!("loading config from file: {}", path);
        let config = std::fs::read_to_string(path)?;
        let config: InternalConfig = toml::from_str(&config)?;

        Ok(Config::from(config))
    }

    pub fn load_from_env() -> Config {
        info!("loading config from ENV");
        let allowed_hosts = std::env::var("ALLOWED_HOSTS")
            .map(|s| s.split(',').map(String::from).collect())
            .unwrap_or_default();

        let blocked_sub_domains = std::env::var("BLOCKED_SUB_DOMAINS")
            .map(|s| s.split(',').map(String::from).collect())
            .unwrap_or_default();

        let master_sig_key = if let Ok(key) = std::env::var("MASTER_SIG_KEY") {
            SigKey::from_hex(&key).expect("invalid master key: not hex or length incorrect")
        } else {
            tracing::warn!("WARNING! generating ephemeral signature key!");
            SigKey::generate()
        };

        let gossip_dns_host = std::env::var("FLY_APP_NAME")
            .map(|app_name| format!("global.{}.internal", app_name))
            .ok();

        let honeycomb_api_key = std::env::var("HONEYCOMB_API_KEY").ok();
        let instance_id = std::env::var("FLY_ALLOC_ID").unwrap_or(Uuid::new_v4().to_string());
        let blocked_ips = std::env::var("BLOCKED_IPS")
            .map(|s| {
                s.split(',')
                    .map(IpAddr::from_str)
                    .filter_map(Result::ok)
                    .collect()
            })
            .unwrap_or_default();

        let portal_host = std::env::var("PORTAL_HOST").unwrap_or("portal.illusiontech.cn".to_string());

        Config {
            allowed_hosts,
            blocked_sub_domains,
            control_port: get_port("CTRL_PORT", 5000),
            remote_port: get_port("PORT", 8080),
            internal_network_port: get_port("NET_PORT", 6000),
            master_sig_key,
            gossip_dns_host,
            honeycomb_api_key,
            instance_id,
            blocked_ips,
            portal_host,
        }
    }
}

fn get_port(var: &'static str, default: u16) -> u16 {
    match std::env::var(var) {
        Ok(port) => port.parse().unwrap_or_else(|_| {
            panic!("invalid port ENV {}={}", var, port);
        }),
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = Config::load_from_env();
        println!("config from env: {:?}", config);
        let config = Config::load_from_file("tests/config.toml").unwrap();
        println!("config from file: {:?}", config);
    }
}
