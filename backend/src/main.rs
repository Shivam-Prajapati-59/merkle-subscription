use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::time::Duration;

mod merkle;
mod model;

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

    // Initialize Solana client
    let rpc_url =
        env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "http://localhost:8899".to_string());
    let keypair_path =
        env::var("SOLANA_KEYPAIR_PATH").unwrap_or_else(|_| "./backend-authority.json".to_string());

    let solana_client = merkle::solana_client::SolanaClient::new(&rpc_url, &keypair_path)?;
    println!("✅ Connected to Solana RPC: {}", rpc_url);

    // Check if config account exists, if not initialize it
    println!("\n🔍 Checking program config...");
    match solana_client.get_current_root().await {
        Ok(current_root) => {
            println!("   ✅ Config account exists");
            println!("   Current root: {}", hex::encode(current_root));
        }
        Err(_) => {
            println!("   ⚠️  Config account not found, initializing...");
            let initial_root = [0u8; 32];
            match solana_client.initialize_config(initial_root).await {
                Ok(sig) => {
                    println!("   ✅ Config initialized! Signature: {}", sig);
                }
                Err(e) => {
                    eprintln!("   ❌ Failed to initialize: {}", e);
                    return Err(e);
                }
            }
        }
    }

    // 1. Build Merkle Tree from database
    let (root_hash, tree, subscriber_data) = merkle::tree::build_tree_from_db(&pool).await?;
    let total_leaves = subscriber_data.len();
    println!("\n🌲 Merkle Tree Built:");
    println!("   Root Hash: {}", root_hash);
    println!("   Total subscribers: {}", total_leaves);

    // 2. Convert hex root to bytes
    let root_bytes: [u8; 32] = hex::decode(&root_hash)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Root must be 32 bytes"))?;

    // 3. Update the merkle root on-chain
    println!("\n📤 Syncing merkle root to Solana...");
    match solana_client.update_merkle_root(root_bytes).await {
        Ok(signature) => {
            println!("✅ Successfully updated on-chain!");

            // 4. Store the transaction in database
            merkle::updatestate::update_merkle_state(
                &pool,
                &root_hash,
                Some(signature.to_string()),
            )
            .await?;
            println!("✅ Saved to database with tx signature");
        }
        Err(e) => {
            eprintln!("❌ Failed to update on-chain: {}", e);
            eprintln!("💡 Tip: If account not initialized, run with --initialize flag");
            eprintln!("        Make sure local validator is running: solana-test-validator");

            // Still save to database but mark as not synced
            merkle::updatestate::update_merkle_state(&pool, &root_hash, None).await?;
        }
    }

    // 5. Verify a user proof (off-chain verification test)
    println!("\n🔐 Testing Proof Verification...");
    if let Some((first_user, expiration)) = subscriber_data.first() {
        println!("   User: {}", first_user);
        println!("   Expiration: {}", expiration);

        if let Some((proof_bytes, index)) =
            merkle::tree::get_proof_for_user(&tree, &subscriber_data, first_user)
        {
            let is_valid = merkle::tree::verify_subscription(
                &root_hash,
                &proof_bytes,
                first_user,
                *expiration,
                index,
                total_leaves,
            )?;

            println!(
                "   Off-chain verification: {}",
                if is_valid { "✓ VALID" } else { "✗ INVALID" }
            );
        }
    }

    // 6. Test tampering detection
    println!("\n🧪 Testing Tampering Detection...");
    if let Some((first_user, _)) = subscriber_data.first() {
        if let Some((proof_bytes, index)) =
            merkle::tree::get_proof_for_user(&tree, &subscriber_data, first_user)
        {
            let fake_expiration = 9999999999i64;
            let is_valid_tamper = merkle::tree::verify_subscription(
                &root_hash,
                &proof_bytes,
                first_user,
                fake_expiration,
                index,
                total_leaves,
            )?;

            println!(
                "   Tampered expiration: {}",
                if is_valid_tamper {
                    "❌ ACCEPTED (Bug!)"
                } else {
                    "✓ REJECTED (Correct)"
                }
            );
        }
    }

    Ok(())
}
