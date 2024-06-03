use std::sync::OnceLock;

use dashmap::DashMap;
use portal_lib::{ClientId, ControlPacket};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

static CLIENT_MANAGER: OnceLock<ClientManager> = OnceLock::new();

pub struct Client {
    pub id: ClientId,
    pub tx: UnboundedSender<ControlPacket>,
    pub rx: UnboundedReceiver<ControlPacket>,
}

#[derive(Default)]
pub struct ClientManager {
    clients: DashMap<ClientId, Client>,
}

impl ClientManager {
    pub fn instance() -> &'static ClientManager {
        CLIENT_MANAGER.get_or_init(ClientManager::default)
    }

    pub fn add(&mut self, client: Client) {
        self.clients.insert(client.id.clone(), client);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_client_manager() {
        let client_manager = ClientManager::instance();
        assert!(client_manager.clients.is_empty());

        let (tx, rx) = unbounded_channel();
        let id: ClientId = "test".into();
        let client = Client { id, tx, rx };

        client_manager.clients.insert(client.id.clone(), client);

        assert_eq!(client_manager.clients.len(), 1);
        assert!(client_manager.clients.contains_key(&"test".into()));

        let client = client_manager.clients.get(&"test".into()).unwrap();
        assert_eq!(client.id, "test".into());
    }
}
