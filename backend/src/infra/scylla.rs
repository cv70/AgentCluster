use crate::config::config::ScyllaConfig;
use crate::datasource::scylladao::dao::ScyllaDao;

use anyhow::Result;
use scylla::{Session, SessionBuilder};

pub async fn new_scylla(c: ScyllaConfig) -> Result<ScyllaDao> {
    let session = SessionBuilder::new()
        .known_node(c.host.as_str(), 9042)
        .user(c.user.as_str(), c.pass.as_str())
        .build()
        .await?;
    
    Ok(ScyllaDao::new(session))
}