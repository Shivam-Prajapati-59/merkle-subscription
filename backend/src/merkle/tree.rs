use anyhow::Result;
use rs_merkle::{Hasher, MerkleTree};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

#[derive(Clone)]
pub struct Sha256Hasher {}

impl Hasher for Sha256Hasher {
    type Hash = [u8; 32];
    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

pub async fn build_tree_from_db(pool: &PgPool) -> Result<(String, MerkleTree<Sha256Hasher>)> {
    // 1. Fetch all addresses from your table
    let rows = sqlx::query_as::<_, (String,)>("SELECT wallet_address FROM subscriber_storage")
        .fetch_all(pool)
        .await?;

    let mut pubkeys: Vec<String> = rows.into_iter().map(|r| r.0).collect();

    if pubkeys.is_empty() {
        return Err(anyhow::anyhow!("No keys found in database"));
    }
    pubkeys.sort();
    pubkeys.dedup();

    // 3. Hash the leaves
    let leaves: Vec<[u8; 32]> = pubkeys
        .iter()
        .map(|pk| Sha256Hasher::hash(pk.as_bytes()))
        .collect();

    // 4. Build the Merkle Tree
    let merkle_tree = MerkleTree::<Sha256Hasher>::from_leaves(&leaves);

    // 5. Get the Root
    let root = merkle_tree
        .root()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate root"))?;

    let root_hex = hex::encode(root);

    Ok((root_hex, merkle_tree))
}
