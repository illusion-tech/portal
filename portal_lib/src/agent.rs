use core::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::SecretKey;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentHandshake {
    agent_id: AgentId,
    auth: Auth,
    version: String,
    local_info: LocalInfo,
    service_info: ServiceInfo,
    encryption: Option<Encryption>,
    heartbeat_interval: u64,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct AgentId(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Auth {
    Key(SecretKey),
    Anonymous,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalInfo {
    ip: IpAddr,
    port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceInfo {
    target_ip: IpAddr,
    target_port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Encryption {
    method: String,
    pub_key: String,
}
