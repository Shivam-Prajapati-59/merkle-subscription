use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub async fn get_db_pool() -> Result<PgPool> {
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set in .env")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to Postgres")?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Test database connection
    let pool = get_db_pool().await?;

    println!("Successfully connected to database!");
    println!("Hello, world!");

    Ok(())
}
