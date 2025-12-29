use anyhow::Result;
use chrono::Utc;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::Keypair;
use sqlx::PgPool;

pub async fn generate_and_store_keys(pool: &PgPool, count: usize) -> Result<()> {
    for _ in 0..count {
        // 1. Generate Keypair
        let kp = Keypair::new();
        let pubkey = kp.pubkey().to_string();

        // 2. Set expiration (e.g., 30 days from now)
        let expiration_ts = (Utc::now().timestamp() + (30 * 24 * 60 * 60)) as i64;

        // 3. Set last updated timestamp (using naive datetime for the DB)
        let last_updated_at = Utc::now().naive_utc();

        // 4. Store in DB
        sqlx::query!(
            "INSERT INTO subscriber_storage (wallet_address, expiration_ts, last_updated_at) VALUES ($1, $2, $3)",
            pubkey,
            expiration_ts,
            last_updated_at
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
