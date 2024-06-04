use std::sync::OnceLock;

use dashmap::{mapref::one::Ref, DashMap};
use portal_lib::{agent::AgentId, ControlPacket};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

static AGENT_MANAGER: OnceLock<AgentManager> = OnceLock::new();

// #[derive(Debug, Clone)]
pub struct Agent {
    pub id: AgentId,
    pub tx: UnboundedSender<ControlPacket>,
    pub rx: UnboundedReceiver<ControlPacket>,
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
        let agent = Agent { id, tx, rx };

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
            tx,
            rx,
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
            tx,
            rx,
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
            tx,
            rx,
        };

        agent_manager.add(agent);

        let nonexistent_id: AgentId = "nonexistent".into();
        let retrieved_agent = agent_manager.get(&nonexistent_id);

        assert!(retrieved_agent.is_none());
    }
}
