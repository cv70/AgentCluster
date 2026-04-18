use crate::config::config::DatabaseConfig;
use crate::datasource::dbdao::DBDao;

use anyhow::Result;
use sqlx::{Pool, Postgres};

pub async fn new_db(c: DatabaseConfig) -> Result<DBDao> {
    let pool = Pool::<Postgres>::connect(&c.postgres_url).await?;
    
    Ok(DBDao::new(pool))
}