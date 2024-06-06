use std::sync::OnceLock;

use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use portal_lib::{agent::AgentId, ControlPacket};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

static AGENT_MANAGER: OnceLock<AgentManager> = OnceLock::new();

/// Tunnel for sending and receiving control packets to and from the agent
pub struct Tunnel {
    /// Sender for sending control packets to the agent
    pub tx: UnboundedSender<ControlPacket>,
    /// Receiver for receiving control packets from the agent
    pub rx: UnboundedReceiver<ControlPacket>,
}

// #[derive(Debug, Clone)]
pub struct Agent {
    pub id: AgentId,
    pub tunnel: Tunnel,
}

#[derive(Default)]
pub struct AgentManager {
    agents: DashMap<AgentId, Agent>,
}

impl AgentManager {
    pub fn instance() -> &'static AgentManager {
        AGENT_MANAGER.get_or_init(AgentManager::default)
    }

    pub fn add(&self, agent: Agent) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn remove(&self, id: &AgentId) {
        self.agents.remove(id);
    }

    pub fn get(&self, id: &AgentId) -> Option<Ref<AgentId, Agent>> {
        self.agents.get(id)
    }

    pub fn get_mut(&self, id: &AgentId) -> Option<RefMut<AgentId, Agent>> {
        self.agents.get_mut(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_agent_manager() {
        let agent_manager = AgentManager::instance();
        assert!(agent_manager.agents.is_empty());

        let (tx, rx) = unbounded_channel();
        let id: AgentId = "test".into();
        let agent = Agent {
            id: id.clone(),
            tunnel: Tunnel { tx, rx },
        };

        agent_manager.agents.insert(agent.id.clone(), agent);

        assert_eq!(agent_manager.agents.len(), 1);
        assert!(agent_manager.agents.contains_key(&"test".into()));

        let agent = agent_manager.agents.get(&"test".into()).unwrap();
        assert_eq!(agent.id, "test".into());
    }

    #[tokio::test]
    async fn test_add_and_remove_agent() {
        let agent_manager = AgentManager::instance();
        assert!(agent_manager.agents.is_empty());

        let (tx, rx) = unbounded_channel();
        let id: AgentId = "test".into();
        let agent = Agent {
            id: id.clone(),
            tunnel: Tunnel { tx, rx },
        };

        agent_manager.add(agent);

        assert_eq!(agent_manager.agents.len(), 1);
        assert!(agent_manager.agents.contains_key(&id));

        agent_manager.remove(&id);

        assert!(agent_manager.agents.is_empty());
        assert!(!agent_manager.agents.contains_key(&id));
    }

    #[tokio::test]
    async fn test_get_agent() {
        let agent_manager = AgentManager::instance();
        assert!(agent_manager.agents.is_empty());

        let (tx, rx) = unbounded_channel();
        let id: AgentId = "test".into();
        let agent = Agent {
            id: id.clone(),
            tunnel: Tunnel { tx, rx },
        };

        agent_manager.add(agent);

        let retrieved_agent = agent_manager.get(&id);

        assert!(retrieved_agent.is_some());
        assert_eq!(retrieved_agent.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_get_nonexistent_agent() {
        let agent_manager = AgentManager::instance();
        assert!(agent_manager.agents.is_empty());

        let (tx, rx) = unbounded_channel();
        let id: AgentId = "test".into();
        let agent = Agent {
            id: id.clone(),
            tunnel: Tunnel { tx, rx },
        };

        agent_manager.add(agent);

        let nonexistent_id: AgentId = "nonexistent".into();
        let retrieved_agent = agent_manager.get(&nonexistent_id);

        assert!(retrieved_agent.is_none());
    }
}
