use std::sync::Arc;

use crate::config::config::AppConfig;
use crate::datasource::{dbdao::dao::DBDao, scylladao::dao::ScyllaDao, vectordao::dao::VectorDao};
use crate::{db, scylla, vector};
use crate::domain::cluster_state_store::ClusterStateStore;
use crate::infra::etcd_client::EtcdClient;

use anyhow::Result;

/// Infrastructure registry holding shared resources
#[derive(Clone)]
pub struct Registry {
    pub db_dao: Arc<DBDao>,
    pub scylla_dao: Arc<ScyllaDao>,
    pub vector_dao: Arc<VectorDao>,
    pub etcd_client: Arc<EtcdClient>,
    pub state_store: Arc<ClusterStateStore>,
}

impl Registry {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let db_dao = db::new_db(config.database.clone()).await?;
        let scylla_dao = scylla::new_scylla(config.scylla.clone()).await?;
        let vector_dao = vector::new_vector(config.vector.clone()).await?;
        let mut etcd_config = config.etcd.clone();
        let etcd_client = EtcdClient::new(etcd_config).await?;
        let state_store = ClusterStateStore::new(config).await?;

        Ok(Self {
            db_dao: Arc::new(db_dao),
            scylla_dao: Arc::new(scylla_dao),
            vector_dao: Arc::new(vector_dao),
            etcd_client: Arc::new(etcd_client),
            state_store: Arc::new(state_store),
        })
    }
}