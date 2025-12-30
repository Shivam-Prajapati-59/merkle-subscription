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
    println!("✅ Successfully connected to database!");

    // Build merkle tree
    let (root_hash, tree, pubkeys) = merkle::tree::build_tree_from_db(&pool).await?;
    let total_leaves = pubkeys.len();
    println!("✅ Merkle Root Hash: {}", root_hash);
    println!("📊 Total leaves in tree: {}", total_leaves);

    // 🔑 Manually specify the user pubkey you want to verify
    let target_user = "BHrpzYrjvZgTcJwJubcUkiuQE2Gh7XtKeRMND5i8FTo2";

    // Try to get proof for the manual user
    if let Some((proof_bytes, index)) =
        merkle::tree::get_proof_for_user(&tree, &pubkeys, target_user)
    {
        println!("\n🔐 Generating proof for: {}", target_user);
        println!(
            "   Index: {}, Proof size: {} bytes",
            index,
            proof_bytes.len()
        );

        // ✅ VERIFY
        let is_valid = merkle::tree::verify_subscription(
            &root_hash,
            &proof_bytes,
            target_user,
            index,
            total_leaves,
        )?;

        println!(
            "\n✅ Verification result: {}",
            if is_valid { "VALID ✓" } else { "INVALID ✗" }
        );
    } else {
        println!("\n❌ User '{}' not found in the tree!", target_user);
        println!("   Available users:");
        for (i, pubkey) in pubkeys.iter().take(5).enumerate() {
            println!("   {}. {}", i + 1, pubkey);
        }
        if pubkeys.len() > 5 {
            println!("   ... and {} more", pubkeys.len() - 5);
        }
    }

    // Test with fake user
    println!("\n🧪 Testing with invalid user...");
    let fake_user = "FakeUser123InvalidPubkey";
    if let Some((proof_bytes, index)) = merkle::tree::get_proof_for_user(&tree, &pubkeys, fake_user)
    {
        let is_valid_fake = merkle::tree::verify_subscription(
            &root_hash,
            &proof_bytes,
            fake_user,
            index,
            total_leaves,
        )?;
        println!(
            "   Fake user verification: {}",
            if is_valid_fake {
                "VALID ✓"
            } else {
                "INVALID ✗"
            }
        );
    } else {
        println!("   ✓ Fake user correctly not found in tree");
    }

    Ok(())
}
