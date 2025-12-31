use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::time::Duration;

mod merkle;
// Assuming your model and tree logic are in these paths
// use merkle::tree;

pub async fn get_db_pool() -> Result<PgPool> {
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL must be set in environment or .env file")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
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

    // 1. Build Merkle Tree
    // Note: pubkeys now contains Vec<(String, i64)> i.e., (Address, Expiration)
    let (root_hash, tree, subscriber_data) = merkle::tree::build_tree_from_db(&pool).await?;
    let total_leaves = subscriber_data.len();
    println!("✅ Merkle Root Hash: {}", root_hash);
    println!("📊 Total leaves in tree: {}", total_leaves);

    // 🔑 User we want to verify
    let target_user = "BHrpzYrjvZgTcJwJubcUkiuQE2Gh7XtKeRMND5i8FTo2";

    // 2. Try to get proof for the target user
    if let Some((proof_bytes, index)) =
        merkle::tree::get_proof_for_user(&tree, &subscriber_data, target_user)
    {
        // Find the expiration time associated with this user in our local data
        let (_, expiration_ts) = subscriber_data[index];

        println!("\n🔐 Generating proof for: {}", target_user);
        println!("   Expiration Timestamp: {}", expiration_ts);
        println!(
            "   Index: {}, Proof size: {} bytes",
            index,
            proof_bytes.len()
        );

        // ✅ VERIFY
        // We now pass the expiration_ts so the verifier can reconstruct the leaf: Hash(PubKey + Expiry)
        let is_valid = merkle::tree::verify_subscription(
            &root_hash,
            &proof_bytes,
            target_user,
            expiration_ts, // Added this argument
            index,
            total_leaves,
        )?;

        println!(
            "\n✅ Verification result: {}",
            if is_valid { "VALID ✓" } else { "INVALID ✗" }
        );
    } else {
        println!("\n❌ User '{}' not found in the tree!", target_user);
        println!("   Available users (first 5):");
        for (i, (pubkey, exp)) in subscriber_data.iter().take(5).enumerate() {
            println!("   {}. {} (Expires: {})", i + 1, pubkey, exp);
        }
    }

    // 🧪 Test with invalid data (Tampering attempt)
    println!("\n🧪 Testing Tampering Attempt (Correct Proof, Wrong Expiration)...");
    if let Some((proof_bytes, index)) =
        merkle::tree::get_proof_for_user(&tree, &subscriber_data, target_user)
    {
        let fake_expiration = 9999999999i64; // A date far in the future
        let is_valid_tamper = merkle::tree::verify_subscription(
            &root_hash,
            &proof_bytes,
            target_user,
            fake_expiration,
            index,
            total_leaves,
        )?;

        println!(
            "   Tampered data verification: {}",
            if is_valid_tamper {
                "FAILED (Security Risk!)"
            } else {
                "SUCCESS (Rejected ✓)"
            }
        );
    }

    Ok(())
}
