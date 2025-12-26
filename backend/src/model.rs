use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]

pub struct SubscriberStorage {
    pub wallet_address: String,
    pub expiration_ts: i64, // BIGINT - Unix timestamp
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MerkleState {
    pub id: i32,
    pub root_hash: String,
    pub is_synced_on_chain: bool,
    pub tx_signature: Option<String>,
    pub created_at: DateTime<Utc>,
}
