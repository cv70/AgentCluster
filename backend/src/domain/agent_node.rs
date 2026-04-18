use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNode {
    pub id: String,
    pub name: String,
    pub status: NodeStatus,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub last_heartbeat: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Pending,
    Running,
    Stopped,
    Terminated,
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Pending
    }
}

impl AgentNode {
    pub fn new(id: String, name: String, capabilities: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            status: NodeStatus::Pending,
            capabilities,
            metadata: HashMap::new(),
            last_heartbeat: now,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        self.updated_at = Utc::now();
    }

    pub fn is_healthy(&self, timeout_seconds: i64) -> bool {
        let timeout = Duration::seconds(timeout_seconds);
        Utc::now() - self.last_heartbeat < timeout
    }
}
