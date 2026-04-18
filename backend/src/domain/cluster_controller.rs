use crate::config::config::AppConfig;
use crate::datasource::{dbdao::DBDao, scylladao::ScyllaDao, vectordao::VectorDao};
use crate::domain::agent_node::{AgentNode, NodeStatus};
use crate::domain::agent_task::{AgentTask, TaskStatus};
use crate::domain::cluster_state_store::ClusterStateStore;
use crate::infra::etcd_client::EtcdClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Cluster Controller - manages nodes and tasks in the cluster
#[derive(Clone)]
pub struct ClusterController {
    db_dao: Arc<DBDao>,
    scylla_dao: Arc<ScyllaDao>,
    vector_dao: Arc<VectorDao>,
    etcd_client: Arc<EtcdClient>,
    state_store: Arc<ClusterStateStore>,
    // In-memory caches for performance
    nodes: Arc<RwLock<HashMap<String, AgentNode>>>,
    tasks: Arc<RwLock<HashMap<String, AgentTask>>>,
}

impl ClusterController {
    pub async fn new(
        db_dao: Arc<DBDao>,
        scylla_dao: Arc<ScyllaDao>,
        vector_dao: Arc<VectorDao>,
        etcd_client: Arc<EtcdClient>,
        state_store: Arc<ClusterStateStore>,
    ) -> Self {
        // Load existing state from persistent storage into memory cache
        let nodes = Arc::new(RwLock::new(HashMap::new()));
        let tasks = Arc::new(RwLock::new(HashMap::new()));
        
        // Load nodes from state store
        if let Ok(stored_nodes) = state_store.list_nodes().await {
            let mut nodes_write = nodes.write().await;
            for node in stored_nodes {
                nodes_write.insert(node.id.clone(), node);
            }
        }
        
        // Load tasks from state store
        if let Ok(stored_tasks) = state_store.list_tasks().await {
            let mut tasks_write = tasks.write().await;
            for task in stored_tasks {
                tasks_write.insert(task.id.clone(), task);
            }
        }
        
        Self {
            db_dao,
            scylla_dao,
            vector_dao,
            etcd_client,
            state_store,
            nodes,
            tasks,
        }
    }

    /// Node management
    pub async fn register_node(&self, mut node: AgentNode) -> Result<(), String> {
        // Generate ID if not provided
        if node.id.is_empty() {
            node.id = Uuid::new_v4().to_string();
        }
        
        // Set timestamps
        let now = Utc::now();
        if node.created_at == DateTime::<Utc>::UNIX_EPOCH {
            node.created_at = now;
        }
        node.updated_at = now;
        
        // Save to persistent storage
        self.state_store.save_node(&node).await.map_err(|e| e.to_string())?;
        
        // Update memory cache
        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(node.id.clone(), node);
        }
        
        info!("Registered node: {}", node.id);
        Ok(())
    }

    pub async fn get_node(&self, node_id: &str) -> Result<Option<AgentNode>, String> {
        // Try memory cache first
        if let Some(node) = self.nodes.read().await.get(node_id) {
            return Ok(Some(node.clone()));
        }
        
        // Fallback to persistent storage
        match self.state_store.get_node(node_id).await {
            Ok(Some(node)) => {
                // Update memory cache
                {
                    let mut nodes = self.nodes.write().await;
                    nodes.insert(node_id.to_string(), node.clone());
                }
                Ok(Some(node))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn list_nodes(&self) -> Result<Vec<AgentNode>, String> {
        // Try memory cache first
        {
            let nodes = self.nodes.read().await;
            if !nodes.is_empty() {
                return Ok(nodes.values().cloned().collect());
            }
        }
        
        // Fallback to persistent storage
        self.state_store.list_nodes().await.map_err(|e| e.to_string())
    }

    pub async fn update_node_status(&self, node_id: &str, status: NodeStatus) -> Result<(), String> {
        // Update in persistent storage first
        self.state_store.update_node_status(node_id, &status.to_string()).await
            .map_err(|e| e.to_string())?;
        
        // Update memory cache
        {
            let mut nodes = self.nodes.write().await;
            if let Some(node) = nodes.get_mut(node_id) {
                node.status = status;
                node.update_heartbeat();
            }
        }
        
        info!("Updated node {} status to {:?}", node_id, status);
        Ok(())
    }

    pub async fn remove_node(&self, node_id: &str) -> Result<(), String> {
        // Remove from persistent storage
        self.state_store.delete_node(node_id).await
            .map_err(|e| e.to_string())?;
        
        // Remove from memory cache
        {
            let mut nodes = self.nodes.write().await;
            nodes.remove(node_id);
        }
        
        info!("Removed node: {}", node_id);
        Ok(())
    }

    /// Task management
    pub async fn submit_task(&self, mut task: AgentTask) -> Result<String, String> {
        // Generate ID if not provided
        if task.id.is_empty() {
            task.id = Uuid::new_v4().to_string();
        }
        
        // Set timestamps
        let now = Utc::now();
        if task.created_at == DateTime::<Utc>::UNIX_EPOCH {
            task.created_at = now;
        }
        task.updated_at = now;
        
        // Validate task
        if task.spec.prompt.is_empty() {
            return Err("Task prompt cannot be empty".to_string());
        }
        
        // Save to persistent storage
        self.state_store.save_task(&task).await.map_err(|e| e.to_string())?;
        
        // Update memory cache
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task.id.clone(), task);
        }
        
        info!("Submitted task: {}", task.id);
        Ok(task.id)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Option<AgentTask>, String> {
        // Try memory cache first
        if let Some(task) = self.tasks.read().await.get(task_id) {
            return Ok(Some(task.clone()));
        }
        
        // Fallback to persistent storage
        match self.state_store.get_task(task_id).await {
            Ok(Some(task)) => {
                // Update memory cache
                {
                    let mut tasks = self.tasks.write().await;
                    tasks.insert(task_id.to_string(), task.clone());
                }
                Ok(Some(task))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn list_tasks(&self) -> Result<Vec<AgentTask>, String> {
        // Try memory cache first
        {
            let tasks = self.tasks.read().await;
            if !tasks.is_empty() {
                return Ok(tasks.values().cloned().collect());
            }
        }
        
        // Fallback to persistent storage
        self.state_store.list_tasks().await.map_err(|e| e.to_string())
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<(), String> {
        // Update in persistent storage first
        self.state_store.update_task_status(task_id, &TaskStatus::Cancelled.to_string()).await
            .map_err(|e| e.to_string())?;
        
        // Update memory cache
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Cancelled;
                task.update_timestamp();
            }
        }
        
        info!("Cancelled task: {}", task_id);
        Ok(())
    }

    // Additional task management methods that might be needed
    pub async fn assign_task(&self, task_id: &str, node_id: &str) -> Result<(), String> {
        // Update in persistent storage
        self.state_store.update_task_assignment(task_id, node_id).await
            .map_err(|e| e.to_string())?;
        
        // Update memory cache
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.assigned_node = Some(node_id.to_string());
                task.status = TaskStatus::Assigned;
                task.update_timestamp();
            }
        }
        
        info!("Assigned task {} to node {}", task_id, node_id);
        Ok(())
    }

    pub async fn start_task(&self, task_id: &str) -> Result<(), String> {
        // Update in persistent storage
        self.state_store.update_task_status(task_id, &TaskStatus::Running.to_string()).await
            .map_err(|e| e.to_string())?;
        
        // Update memory cache
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Running;
                task.started_at = Some(Utc::now());
                task.update_timestamp();
            }
        }
        
        info!("Started task: {}", task_id);
        Ok(())
    }

    pub async fn complete_task(&self, task_id: &str, result: String) -> Result<(), String> {
        // Update in persistent storage
        self.state_store.update_task_status_and_result(task_id, &TaskStatus::Completed.to_string(), Some(result)).await
            .map_err(|e| e.to_string())?;
        
        // Update memory cache
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(Utc::now());
                task.result = Some(result);
                task.update_timestamp();
            }
        }
        
        info!("Completed task: {}", task_id);
        Ok(())
    }

    pub async fn fail_task(&self, task_id: &str, error: String) -> Result<(), String> {
        // Update in persistent storage
        self.state_store.update_task_status_and_error(task_id, &TaskStatus::Failed.to_string(), Some(error)).await
            .map_err(|e| e.to_string())?;
        
        // Update memory cache
        {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.status = TaskStatus::Failed;
                task.completed_at = Some(Utc::now());
                task.error = Some(error);
                task.update_timestamp();
            }
        }
        
        info!("Failed task: {}", task_id);
        Ok(())
    }

    /// Health check and metrics
    pub async fn get_cluster_status(&self) -> Result<ClusterStatus, String> {
        let nodes = self.list_nodes().await?;
        let tasks = self.list_tasks().await?;
        
        let online_nodes = nodes.iter()
            .filter(|n| n.is_healthy(90)) // 90 second heartbeat timeout
            .count();
        
        let pending_tasks = tasks.iter()
            .filter(|t| matches!(t.status, TaskStatus::Pending | TaskStatus::Assigned | TaskStatus::Running))
            .count();
        
        Ok(ClusterStatus {
            total_nodes: nodes.len(),
            online_nodes,
            total_tasks: tasks.len(),
            pending_tasks,
        })
    }

    pub async fn get_cluster_metrics(&self) -> Result<ClusterMetrics, String> {
        // In a real implementation, we'd collect actual metrics from historical data
        // For now, we'll return placeholder values
        Ok(ClusterMetrics {
            avg_task_duration_ms: 0,
            tasks_completed_today: 0,
            nodes_joined_today: 0,
            nodes_left_today: 0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ClusterStatus {
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub total_tasks: usize,
    pub pending_tasks: usize,
}

#[derive(Debug, Clone)]
pub struct ClusterMetrics {
    pub avg_task_duration_ms: u64,
    pub tasks_completed_today: u64,
    pub nodes_joined_today: u64,
    pub nodes_left_today: u64,
}