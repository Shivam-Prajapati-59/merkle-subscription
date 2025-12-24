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
    let clock = Clock::get()?;

    // 1. Check expiration FIRST.
    require!(
        expiration > clock.unix_timestamp,
        SubscriptionError::SubscriptionExpired
    );

    // 2. Leaf Hash
    // Input Length: 32 (Pubkey) + 8 (i64) = 40 bytes.
    let expiration_bytes = expiration.to_le_bytes();
    let leaf = hashv(&[&user_key.to_bytes(), &expiration_bytes]).to_bytes();

    // 3. Verify Proof
    let mut current_hash = leaf;
    for node in proof {
        current_hash = if current_hash <= node {
            hashv(&[&current_hash, &node]).to_bytes()
        } else {
            hashv(&[&node, &current_hash]).to_bytes()
        };
    }

    // 4. Verify Root
    require!(
        current_hash == ctx.accounts.config.merkle_root,
        SubscriptionError::InvalidProof
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
