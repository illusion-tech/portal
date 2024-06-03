use std::sync::OnceLock;

use dashmap::{mapref::one::Ref, DashMap};
use portal_lib::{ClientId, ControlPacket};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

static CLIENT_MANAGER: OnceLock<ClientManager> = OnceLock::new();

// #[derive(Debug, Clone)]
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

    pub fn add(&self, client: Client) {
        self.clients.insert(client.id.clone(), client);
    }

    pub fn remove(&self, id: &ClientId) {
        self.clients.remove(id);
    }

    pub fn get(&self, id: &ClientId) -> Option<Ref<ClientId, Client>> {
        self.clients.get(id)
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

    #[tokio::test]
    async fn test_add_and_remove_client() {
        let client_manager = ClientManager::instance();
        assert!(client_manager.clients.is_empty());

        let (tx, rx) = unbounded_channel();
        let id: ClientId = "test".into();
        let client = Client {
            id: id.clone(),
            tx,
            rx,
        };

        client_manager.add(client);

        assert_eq!(client_manager.clients.len(), 1);
        assert!(client_manager.clients.contains_key(&id));

        client_manager.remove(&id);

        assert!(client_manager.clients.is_empty());
        assert!(!client_manager.clients.contains_key(&id));
    }

    #[tokio::test]
    async fn test_get_client() {
        let client_manager = ClientManager::instance();
        assert!(client_manager.clients.is_empty());

        let (tx, rx) = unbounded_channel();
        let id: ClientId = "test".into();
        let client = Client {
            id: id.clone(),
            tx,
            rx,
        };

        client_manager.add(client);

        let retrieved_client = client_manager.get(&id);

        assert!(retrieved_client.is_some());
        assert_eq!(retrieved_client.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_get_nonexistent_client() {
        let client_manager = ClientManager::instance();
        assert!(client_manager.clients.is_empty());

        let (tx, rx) = unbounded_channel();
        let id: ClientId = "test".into();
        let client = Client {
            id: id.clone(),
            tx,
            rx,
        };

        client_manager.add(client);

        let nonexistent_id: ClientId = "nonexistent".into();
        let retrieved_client = client_manager.get(&nonexistent_id);

        assert!(retrieved_client.is_none());
    }
}
