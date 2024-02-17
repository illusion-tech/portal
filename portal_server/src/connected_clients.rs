use super::*;
use dashmap::DashMap;
use std::fmt::Formatter;

#[derive(Clone)]
pub struct ConnectedClient {
    pub id: ClientId,
    pub host: String,
    pub is_anonymous: bool,
    pub tx: UnboundedSender<ControlPacket>,
}

impl std::fmt::Debug for ConnectedClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectedClient")
            .field("id", &self.id)
            .field("sub", &self.host)
            .field("anon", &self.is_anonymous)
            .finish()
    }
}

pub struct Connections {
    clients: Arc<DashMap<ClientId, ConnectedClient>>,
    hosts: Arc<DashMap<String, ConnectedClient>>,
}

impl Default for Connections {
    fn default() -> Self {
        Self {
            clients: Arc::new(DashMap::new()),
            hosts: Arc::new(DashMap::new()),
        }
    }
}

impl Connections {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_host(client: &ConnectedClient) {
        get_connections()
            .hosts
            .insert(client.host.clone(), client.clone());
    }

    pub fn remove(client: &ConnectedClient) {
        client.tx.close_channel();

        let connections = get_connections();
        // ensure another client isn't using this host
        if connections
            .hosts
            .get(&client.host)
            .map_or(false, |c| c.id == client.id)
        {
            tracing::debug!("dropping sub-domain: {}", &client.host);
            connections.hosts.remove(&client.host);
        };

        connections.clients.remove(&client.id);
        tracing::debug!("rm client: {}", &client.id);

        // // drop all the streams
        // // if there are no more tunnel clients
        // if CONNECTIONS.clients.is_empty() {
        //     let mut streams = ACTIVE_STREAMS.;
        //     for (_, stream) in streams.drain() {
        //         stream.tx.close_channel();
        //     }
        // }
    }

    pub fn client_for_host(host: &String) -> Option<ClientId> {
        get_connections().hosts.get(host).map(|c| c.id.clone())
    }

    pub fn get(client_id: &ClientId) -> Option<ConnectedClient> {
        get_connections()
            .clients
            .get(client_id)
            .map(|c| c.value().clone())
    }

    pub fn find_by_host(host: &String) -> Option<ConnectedClient> {
        get_connections().hosts.get(host).map(|c| c.value().clone())
    }

    pub fn add(client: ConnectedClient) {
        let connections = get_connections();
        connections
            .clients
            .insert(client.id.clone(), client.clone());
        connections.hosts.insert(client.host.clone(), client);
    }
}
