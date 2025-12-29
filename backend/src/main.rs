use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::time::Duration;

mod merkle;
mod model;

pub async fn get_db_pool() -> Result<PgPool> {
    // 1. Ensure the URL is available
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL must be set in environment or .env file")?;

    // 2. Configure the pool with better defaults for stability
    let pool = PgPoolOptions::new()
        .max_connections(5)
        // Add a timeout so the app doesn't hang forever if the DB is down
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .context("Failed to connect to Postgres. Ensure the service is running.")?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().context("Failed to load .env file")?;

    let pool = get_db_pool().await?;

    // 4. Verify the connection is actually alive with a simple query
    sqlx::query("SELECT 1").execute(&pool).await?;

    println!("✅ Successfully connected to database!");

    // Generate and store 10 test keypairs
    println!("🔑 Generating keypairs...");
    merkle::generator::generate_and_store_keys(&pool, 10).await?;
    println!("✅ Generated and stored 10 keypairs!");

    // Build merkle tree and get root hash
    println!("\n🌲 Building Merkle tree...");
    let (root_hash, _tree) = merkle::tree::build_tree_from_db(&pool).await?;
    println!("✅ Merkle Root Hash: {}", root_hash);

    Ok(())
}
