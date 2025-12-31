use crate::error::SubscriptionError;
use crate::state::SubscriptionConfig;
use anchor_lang::prelude::*;
use rs_merkle::{Hasher, MerkleProof};
use sha2::{Digest, Sha256};

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

pub fn verify_subscription(
    ctx: Context<VerifySubscription>,
    proof_bytes: Vec<u8>,
    expiration: i64,
    leaf_index: usize,
    total_leaves: usize,
) -> Result<()> {
    let user_key = ctx.accounts.user.key();
    let clock = Clock::get()?;

    // 1. Check expiration FIRST
    require!(
        expiration > clock.unix_timestamp,
        SubscriptionError::SubscriptionExpired
    );

    // 2. Reconstruct leaf: Hash(pubkey_bytes + expiration_bytes)
    let mut leaf_data = Vec::with_capacity(40);
    leaf_data.extend_from_slice(&user_key.to_bytes());
    leaf_data.extend_from_slice(&expiration.to_le_bytes());
    let leaf = Sha256Hasher::hash(&leaf_data);

    // 3. Parse the merkle proof
    let proof = MerkleProof::<Sha256Hasher>::try_from(proof_bytes.as_slice())
        .map_err(|_| SubscriptionError::InvalidProof)?;

    // 4. Verify proof against stored root
    let is_valid = proof.verify(
        ctx.accounts.config.merkle_root,
        &[leaf_index],
        &[leaf],
        total_leaves,
    );

    require!(is_valid, SubscriptionError::InvalidProof);

    msg!("Verification successful for user: {}", user_key);
    Ok(())
}

#[derive(Accounts)]
pub struct VerifySubscription<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, SubscriptionConfig>,
    pub user: Signer<'info>,
}
