use crate::error::SubscriptionError;
use crate::state::SubscriptionConfig;
use anchor_lang::prelude::*;
use solana_program::hash::hashv;

pub fn verify_subscription(
    ctx: Context<VerifySubscription>,
    proof: Vec<[u8; 32]>,
    expiration: i64,
) -> Result<()> {
    let user_key = ctx.accounts.user.key();

    // 1. Recreate the Leaf hash using SHA256
    // Format: hash(wallet_address + expiration)
    let leaf = hashv(&[&user_key.to_bytes(), &expiration.to_le_bytes()]).to_bytes();

    // 2. Verify the proof against the stored root
    let mut current_hash = leaf;
    for node in proof {
        // Standard Merkle path hashing (sorting nodes to maintain order)
        current_hash = if current_hash <= node {
            hashv(&[&current_hash, &node]).to_bytes()
        } else {
            hashv(&[&node, &current_hash]).to_bytes()
        };
    }

    // 3. Compare with stored Merkle Root
    require!(
        current_hash == ctx.accounts.config.merkle_root,
        SubscriptionError::InvalidProof
    );

    // 4. Expiration check
    let clock = Clock::get()?;
    require!(
        expiration > clock.unix_timestamp,
        SubscriptionError::SubscriptionExpired
    );

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
