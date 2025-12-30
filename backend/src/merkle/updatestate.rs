use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;

pub async fn update_merkle_state(
    pool: &PgPool,
    root_hex: &str,
    tx_signature: Option<String>,
) -> Result<()> {
    let is_synced = tx_signature.is_some();
    let created_at = Utc::now().naive_utc();

    // Store the updated RootHash into the db
    sqlx::query!(
        "INSERT INTO merkle_state (root_hash, is_synced_on_chain, tx_signature, created_at) 
         VALUES ($1, $2, $3, $4)",
        root_hex,
        is_synced,
        tx_signature,
        created_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update existing merkle state with transaction signature
pub async fn sync_merkle_state_on_chain(
    pool: &PgPool,
    root_hash: &str,
    tx_signature: &str,
) -> Result<()> {
    sqlx::query!(
        "UPDATE merkle_state 
         SET is_synced_on_chain = TRUE, tx_signature = $1 
         WHERE root_hash = $2",
        tx_signature,
        root_hash
    )
    .execute(pool)
    .await?;

    Ok(())
}
