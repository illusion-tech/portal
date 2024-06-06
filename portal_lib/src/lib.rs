pub mod agent;
mod control_packet;

pub use control_packet::ControlPacket;
pub use control_packet::PING_INTERVAL;

use base64::{engine::general_purpose, Engine as _};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct SecretKey(pub String);
impl SecretKey {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        Self(
            std::iter::repeat(())
                .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
                .take(22)
                .collect::<String>(),
        )
    }

    pub fn client_id(&self) -> ClientId {
        ClientId(general_purpose::STANDARD_NO_PAD.encode(sha2::Sha256::digest(self.0.as_bytes())))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ReconnectToken(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ServerHello {
    Success {
        sub_domain: String,
        hostname: String,
        client_id: ClientId,
    },
    SubDomainInUse,
    InvalidSubDomain,
    AuthFailed,
    Error(String),
}

impl ServerHello {
    #[allow(unused)]
    pub fn random_domain() -> String {
        let mut rng = rand::thread_rng();
        std::iter::repeat(())
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .take(8)
            .collect::<String>()
            .to_lowercase()
    }

    #[allow(unused)]
    pub fn prefixed_random_domain(prefix: &str) -> String {
        prefix.into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientHello {
    /// deprecated: just send some garbage
    id: ClientId,
    pub sub_domain: Option<String>,
    pub client_type: ClientType,
    pub reconnect_token: Option<ReconnectToken>,
}

impl ClientHello {
    pub fn generate(sub_domain: Option<String>, typ: ClientType) -> Self {
        ClientHello {
            id: ClientId::generate(),
            client_type: typ,
            sub_domain,
            reconnect_token: None,
        }
    }

    pub fn reconnect(reconnect_token: ReconnectToken) -> Self {
        ClientHello {
            id: ClientId::generate(),
            sub_domain: None,
            client_type: ClientType::Anonymous,
            reconnect_token: Some(reconnect_token),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientType {
    Auth { key: SecretKey },
    Anonymous,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ClientId(String);

impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        ClientId(s.into())
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl ClientId {
    pub fn generate() -> Self {
        let mut id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut id);
        ClientId(general_purpose::URL_SAFE_NO_PAD.encode(id))
    }

    pub fn safe_id(self) -> ClientId {
        ClientId(general_purpose::STANDARD.encode(sha2::Sha256::digest(self.0.as_bytes())))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamId([u8; 8]);

impl StreamId {
    pub fn generate() -> StreamId {
        let mut id = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut id);
        StreamId(id)
    }
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "stream_{}",
            general_purpose::URL_SAFE_NO_PAD.encode(self.0)
        )
    }
}
