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
) -> Result<(String, MerkleTree<Sha256Hasher>, Vec<String>)> {
    let rows = sqlx::query_as::<_, (String,)>("SELECT wallet_address FROM subscriber_storage")
        .fetch_all(pool)
        .await?;

    let mut pubkeys: Vec<String> = rows.into_iter().map(|r| r.0).collect();
    if pubkeys.is_empty() {
        return Err(anyhow::anyhow!("No keys found in database"));
    }

    pubkeys.sort();
    pubkeys.dedup();

    let leaves: Vec<[u8; 32]> = pubkeys
        .iter()
        .map(|pk| Sha256Hasher::hash(pk.as_bytes()))
        .collect();

    let merkle_tree = MerkleTree::<Sha256Hasher>::from_leaves(&leaves);
    let root = merkle_tree
        .root()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate root"))?;

    Ok((hex::encode(root), merkle_tree, pubkeys))
}

/// Returns (Serialized Proof Bytes, Leaf Index)
pub fn get_proof_for_user(
    tree: &MerkleTree<Sha256Hasher>,
    pubkeys: &[String],
    user_pubkey: &str,
) -> Option<(Vec<u8>, usize)> {
    let index = pubkeys.iter().position(|pk| pk == user_pubkey)?;
    let proof = tree.proof(&[index]);

    // Serializing to bytes is the standard way to pass rs_merkle proofs
    Some((proof.to_bytes(), index))
}

pub fn verify_subscription(
    root_hex: &str,
    proof_bytes: &[u8],
    user_pubkey: &str,
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

    // 3. Hash the leaf
    let leaf = Sha256Hasher::hash(user_pubkey.as_bytes());

    // 4. Verify
    Ok(proof.verify(root, &[index], &[leaf], total_subscribers))
}
