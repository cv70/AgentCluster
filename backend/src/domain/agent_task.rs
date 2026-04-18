use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub assigned_node: Option<String>,
    pub requirements: Vec<String>,
    pub data: HashMap<String, String>,
    pub result: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

impl AgentTask {
    pub fn new(id: String, name: String, requirements: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            status: TaskStatus::Pending,
            assigned_node: None,
            requirements,
            data: HashMap::new(),
            result: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn assign_to_node(&mut self, node_id: String) {
        self.assigned_node = Some(node_id);
        self.status = TaskStatus::Assigned;
        self.update_timestamp();
    }

    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.update_timestamp();
    }

    pub fn complete(&mut self, result: String) {
        self.status = TaskStatus::Completed;
        self.result = Some(result);
        self.update_timestamp();
    }

    pub fn fail(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.result = Some(error);
        self.update_timestamp();
    }
}
