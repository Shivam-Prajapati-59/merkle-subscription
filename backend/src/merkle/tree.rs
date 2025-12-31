use anyhow::{Context, Result};
use rs_merkle::{Hasher, MerkleProof, MerkleTree};
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

pub async fn build_tree_from_db(
    pool: &PgPool,
) -> Result<(String, MerkleTree<Sha256Hasher>, Vec<(String, i64)>)> {
    // 1. Fetch both wallet and expiration
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT wallet_address, expiration_ts FROM subscriber_storage",
    )
    .fetch_all(pool)
    .await?;

    let mut subscribers = rows;
    if subscribers.is_empty() {
        return Err(anyhow::anyhow!("No subscribers found in database"));
    }

    // Sort by wallet_address to keep the tree deterministic
    subscribers.sort_by(|a, b| a.0.cmp(&b.0));

    // 2. Generate Leaves: Hash(PubKey_BYTES + Expiration)
    // ⚠️ CRITICAL: Must decode base58 pubkey to 32 bytes (matches Solana's user_key.to_bytes())
    let leaves: Vec<[u8; 32]> = subscribers
        .iter()
        .map(|(pk_str, exp)| {
            // Decode base58 pubkey to 32 bytes
            let pubkey_bytes = bs58::decode(pk_str)
                .into_vec()
                .expect("Invalid base58 pubkey in database");

            if pubkey_bytes.len() != 32 {
                panic!("Pubkey must be exactly 32 bytes");
            }

            let mut payload = Vec::with_capacity(40);
            payload.extend_from_slice(&pubkey_bytes);
            payload.extend_from_slice(&exp.to_le_bytes());
            Sha256Hasher::hash(&payload)
        })
        .collect();

    let merkle_tree = MerkleTree::<Sha256Hasher>::from_leaves(&leaves);
    let root = merkle_tree
        .root()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate root"))?;

    Ok((hex::encode(root), merkle_tree, subscribers))
}

/// Returns (Serialized Proof Bytes, Leaf Index)
pub fn get_proof_for_user(
    tree: &MerkleTree<Sha256Hasher>,
    subscribers: &[(String, i64)],
    user_pubkey: &str,
) -> Option<(Vec<u8>, usize)> {
    let index = subscribers.iter().position(|(pk, _)| pk == user_pubkey)?;
    let proof = tree.proof(&[index]);

    Some((proof.to_bytes(), index))
}

pub fn verify_subscription(
    root_hex: &str,
    proof_bytes: &[u8],
    user_pubkey: &str,
    expiration_ts: i64,
    index: usize,
    total_subscribers: usize,
) -> Result<bool> {
    // 1. Decode root
    let root_vec = hex::decode(root_hex).context("Invalid root hex")?;
    let root: [u8; 32] = root_vec
        .try_into()
        .map_err(|_| anyhow::anyhow!("Root must be 32 bytes"))?;

    // 2. Parse proof
    let proof = MerkleProof::<Sha256Hasher>::try_from(proof_bytes)
        .map_err(|_| anyhow::anyhow!("Invalid proof format"))?;

    // 3. Reconstruct the SAME leaf: Hash(PubKey_BYTES + Expiration)
    // ⚠️ CRITICAL: Decode base58 pubkey to bytes (matches on-chain user_key.to_bytes())
    let pubkey_bytes = bs58::decode(user_pubkey)
        .into_vec()
        .context("Invalid base58 pubkey")?;

    if pubkey_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Pubkey must be 32 bytes"));
    }

    let mut payload = Vec::with_capacity(40);
    payload.extend_from_slice(&pubkey_bytes);
    payload.extend_from_slice(&expiration_ts.to_le_bytes());
    let leaf = Sha256Hasher::hash(&payload);

    // 4. Verify
    Ok(proof.verify(root, &[index], &[leaf], total_subscribers))
}
