// Application configuration
use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub etcd: EtcdConfig,
    pub security: SecurityConfig,
    pub scylla: ScyllaConfig,
    pub vector: VectorConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub scylla_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EtcdConfig {
    pub endpoints: Vec<String>,
    pub username: String,
    pub password: String,
    pub prefix: String,
    pub timeout_secs: u64,
}

impl Default for EtcdConfig {
    fn default() -> Self {
        Self {
            endpoints: vec!["http://localhost:2379".to_string()],
            username: "".to_string(),
            password: "".to_string(),
            prefix: "/agentcluster".to_string(),
            timeout_secs: 5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScyllaConfig {
    pub hosts: Vec<String>,
    pub username: String,
    pub password: String,
    pub keyspace: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VectorConfig {
    pub host: String,
    pub port: u16,
    pub api_key: String,
}

impl AppConfig {
    pub fn load_from_path(path: &str) -> Result<Self> {
        let raw = fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&raw)?;
        Ok(config)
    }

    pub fn load() -> Result<Self> {
        Self::load_from_path("config.yaml")
    }
}

pub fn parse_config_path_from_args<I>(args: I) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--config" {
            return iter.next();
        }
    }
    None
}
