use core::net::IpAddr;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// The handshake message structure used by an Agent to communicate with the Server.
#[derive(Builder, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[builder(try_setter, setter(into))]
pub struct AgentHandshake {
    /// Unique identifier for the Agent.
    agent_id: AgentId,
    /// Name of the Agent.
    agent_name: Option<String>,
    /// Authentication information for the Agent, can be a key or anonymous.
    auth: Auth,
    /// The version of the Agent software.
    version: String,
    /// Local network information of the Agent.
    local_info: LocalInfo,
    /// Information about the service the Agent is providing or connecting to.
    service_info: ServiceInfo,
    /// Encryption information for secure communication.
    encryption: Option<Encryption>,
    /// Interval in milliseconds for the Agent to send a heartbeat message.
    heartbeat_interval: u32,
    /// Timestamp of the handshake message.
    timestamp: u64,
}

impl AgentHandshake {
    pub fn builder() -> AgentHandshakeBuilder {
        AgentHandshakeBuilder::default()
    }
}

/// Unique identifier for an Agent.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct AgentId(String);

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        AgentId(s.into())
    }
}

/// Authentication information for the Agent.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Auth {
    /// Authentication using a secret key.
    Key(String),
    /// Anonymous authentication.
    Anonymous,
}

/// Local network information of the Agent.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LocalInfo {
    /// Local IP address of the Agent.
    ip: IpAddr,
    /// Local port the Agent is listening on.
    port: u16,
}

/// Information about the service the Agent is providing or connecting to.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ServiceInfo {
    /// IP address of the target service.
    target_ip: IpAddr,
    /// Port of the target service.
    target_port: u16,
}

/// Encryption information for secure communication.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Encryption {
    /// Encryption method used.
    method: String,
    /// Public key used for encryption.
    key: String,
}

#[cfg(test)]
mod tests {
    use core::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_agent_handshake_serialization() {
        let agent_id = AgentId("agent-123".to_string());
        let agent_name = None;
        let auth = Auth::Key("secret-key".to_string());
        let version = "1.0.0".to_string();
        let local_info = LocalInfo {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
        };
        let service_info = ServiceInfo {
            target_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)),
            target_port: 9000,
        };
        let encryption = Some(Encryption {
            method: "AES-256".to_string(),
            key: "encryption-key".to_string(),
        });
        let heartbeat_interval = 5000;
        let timestamp = 1631234567890;

        let handshake = AgentHandshake {
            agent_id,
            agent_name,
            auth,
            version,
            local_info,
            service_info,
            encryption,
            heartbeat_interval,
            timestamp,
        };

        let serialized = serde_json::to_string(&handshake).unwrap();
        let deserialized: AgentHandshake = serde_json::from_str(&serialized).unwrap();

        println!("{} - {}", serialized.len(), serialized);
        println!("{:?}", deserialized);

        assert_eq!(handshake, deserialized);
    }

    #[test]
    fn test_agent_handshake_builder() {
        let handshake = AgentHandshake::builder()
            .agent_id(AgentId("agent-123".to_string()))
            .auth(Auth::Key("secret-key".to_string()))
            .version("1.0.0".to_string())
            .local_info(LocalInfo {
                ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                port: 8080,
            })
            .service_info(ServiceInfo {
                target_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)),
                target_port: 9000,
            })
            .encryption(Some(Encryption {
                method: "AES-256".to_string(),
                key: "encryption-key".to_string(),
            }))
            .heartbeat_interval(5000u32)
            .timestamp(1631234567890u64)
            .build()
            .unwrap();

        assert_eq!(handshake.agent_id.0, "agent-123");
    }
}
