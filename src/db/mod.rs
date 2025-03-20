use crate::prelude::*;
use sqlx::PgPool;
use std::env;

/// Create a new database pool
pub async fn create_db_pool() -> Result<PgPool, OddbotError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPool::connect(&database_url)
        .await
        .map_err(OddbotError::Database)
}
