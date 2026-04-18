use std::sync::Arc;

use crate::config::config::AppConfig;
use crate::domain::agent_node::AgentNode;
use crate::domain::agent_task::AgentTask;
use crate::infra::etcd_client::EtcdClient;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{info, warn};

/// Etcd-based cluster state store for persistent storage of cluster state
#[derive(Clone)]
pub struct ClusterStateStore {
    etcd_client: Arc<EtcdClient>,
    nodes_prefix: String,
    tasks_prefix: String,
}

impl ClusterStateStore {
    pub async fn new(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut etcd_config = config.etcd.clone();
        let etcd_client = EtcdClient::new(etcd_config).await?;
        
        let prefix = if etcd_config.prefix.is_empty() {
            "/agentcluster".to_string()
        } else {
            etcd_config.prefix.clone()
        };
        
        let nodes_prefix = format!("{}/nodes", prefix);
        let tasks_prefix = format!("{}/tasks", prefix);
        
        // Ensure prefixes exist (in etcd, this happens automatically when we create keys)
        
        info!("ClusterStateStore initialized with etcd endpoints: {:?}", etcd_config.endpoints);
        
        Ok(Self {
            etcd_client: Arc::new(etcd_client),
            nodes_prefix,
            tasks_prefix,
        })
    }

    /// Save a node to etcd
    pub async fn save_node(&self, node: &AgentNode) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("{}/{}", self.nodes_prefix, node.id);
        let value = serde_json::to_vec(node)?;
        self
            .etcd_client
            .client
            .put(&key, value, None)
            .await?;
        info!("Saved node {} to etcd", node.id);
        Ok(())
    }

    /// Get a node from etcd by ID
    pub async fn get_node(&self, node_id: &str) -> Result<Option<AgentNode>, Box<dyn std::error::Error>> {
        let key = format!("{}/{}", self.nodes_prefix, node_id);
        let resp = self.etcd_client.client.get(&key, None).await?;
        
        if resp.kvs().is_empty() {
            return Ok(None);
        }
        
        let value = String::from_utf8(resp.kvs()[0].value().to_vec())?;
        let node: AgentNode = serde_json::from_str(&value)?;
        Ok(Some(node))
    }

    /// Delete a node from etcd
    pub async fn delete_node(&self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("{}/{}", self.nodes_prefix, node_id);
        self.etcd_client.client.delete(&key, None).await?;
        info!("Deleted node {} from etcd", node_id);
        Ok(())
    }

    /// Update node status in etcd
    pub async fn update_node_status(&self, node_id: &str, status: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get existing node
        let mut node = match self.get_node(node_id).await? {
            Some(n) => n,
            None => return Err(format!("Node {} not found", node_id).into()),
        };
        
        // Update status
        node.status = status.to_string();
        node.update_heartbeat();
        
        // Save back to etcd
        self.save_node(&node).await?;
        info!("Updated node {} status to {}", node_id, status);
        Ok(())
    }

    /// Save a task to etcd
    pub async fn save_task(&self, task: &AgentTask) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("{}/{}", self.tasks_prefix, task.id);
        let value = serde_json::to_vec(task)?;
        self
            .etcd_client
            .client
            .put(&key, value, None)
            .await?;
        info!("Saved task {} to etcd", task.id);
        Ok(())
    }

    /// Get a task from etcd by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Option<AgentTask>, Box<dyn std::error::Error>> {
        let key = format!("{}/{}", self.tasks_prefix, task_id);
        let resp = self.etcd_client.client.get(&key, None).await?;
        
        if resp.kvs().is_empty() {
            return Ok(None);
        }
        
        let value = String::from_utf8(resp.kvs()[0].value().to_vec())?;
        let task: AgentTask = serde_json::from_str(&value)?;
        Ok(Some(task))
    }

    /// Delete a task from etcd
    pub async fn delete_task(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("{}/{}", self.tasks_prefix, task_id);
        self.etcd_client.delete(&key, None).await?;
        info!("Deleted task {} from etcd", task_id);
        Ok(())
    }

    /// Update task status in etcd
    pub async fn update_task_status(&self, task_id: &str, status: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get existing task
        let mut task = match self.get_task(task_id).await? {
            Some(t) => t,
            None => return Err(format!("Task {} not found", task_id).into()),
        };
        
        // Update status
        task.status = status.to_string();
        task.update_timestamp();
        
        // Save back to etcd
        self.save_task(&task).await?;
        info!("Updated task {} status to {}", task_id, status);
        Ok(())
    }

    /// Update task assignment in etcd
    pub async fn update_task_assignment(&self, task_id: &str, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get existing task
        let mut task = match self.get_task(task_id).await? {
            Some(t) => t,
            None => return Err(format!("Task {} not found", task_id).into()),
        };
        
        // Update assignment
        task.assigned_node = Some(node_id.to_string());
        task.update_timestamp();
        
        // Save back to etcd
        self.save_task(&task).await?;
        info!("Assigned task {} to node {}", task_id, node_id);
        Ok(())
    }

    /// Update task status and result in etcd
    pub async fn update_task_status_and_result(
        &self,
        task_id: &str,
        status: &str,
        result: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get existing task
        let mut task = match self.get_task(task_id).await? {
            Some(t) => t,
            None => return Err(format!("Task {} not found", task_id).into()),
        };
        
        // Update status and result
        task.status = status.to_string();
        task.result = result;
        task.update_timestamp();
        
        // Save back to etcd
        self.save_task(&task).await?;
        info!("Updated task {} status to {} with result", task_id, status);
        Ok(())
    }

    /// Update task status and error in etcd
    pub async fn update_task_status_and_error(
        &self,
        task_id: &str,
        status: &str,
        error: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get existing task
        let mut task = match self.get_task(task_id).await? {
            Some(t) => t,
            None => return Err(format!("Task {} not found", task_id).into()),
        };
        
        // Update status and error
        task.status = status.to_string();
        task.error = error;
        task.update_timestamp();
        
        // Save back to etcd
        self.save_task(&task).await?;
        info!("Updated task {} status to {} with error", task_id, status);
        Ok(())
    }

    /// List all nodes from etcd
    pub async fn list_nodes(&self) -> Result<Vec<AgentNode>, Box<dyn std::error::Error>> {
        let mut nodes = Vec::new();
        
        // Get all keys under nodes prefix
        if let Some(range) = self
            .etcd_client
            .range(&self.nodes_prefix, None)
            .await?
        {
            for (_, value) in range.kvs {
                if let Ok(node) = serde_json::from_slice::<AgentNode>(&value) {
                    nodes.push(node);
                }
            }
        }
        
        Ok(nodes)
    }

    /// List all tasks from etcd
    pub async fn list_tasks(&self) -> Result<Vec<AgentTask>, Box<dyn std::error::Error>> {
        let mut tasks = Vec::new();
        
        // Get all keys under tasks prefix
        if let Some(range) = self
            .etcd_client
            .range(&self.tasks_prefix, None)
            .await?
        {
            for (_, value) in range.kvs {
                if let Ok(task) = serde_json::from_slice::<AgentTask>(&value) {
                    tasks.push(task);
                }
            }
        }
        
        Ok(tasks)
    }
}