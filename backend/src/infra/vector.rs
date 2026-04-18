use crate::config::config::VectorConfig;
use crate::datasource::vectordao::dao::VectorDao;

use anyhow::Result;
use qdrant_client::Qdrant;

pub async fn new_vector(c: VectorConfig) -> Result<VectorDao> {
    let client = Qdrant::from_url(&format!("http://{}:{}", c.host, c.port))
        .api_key(&c.api_key)
        .build()?;
    
    Ok(VectorDao::new(client))
}