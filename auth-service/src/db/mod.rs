pub mod migrations;

use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;

    // Run migrations
    migrations::run_migrations(&pool).await?;

    Ok(pool)
}
