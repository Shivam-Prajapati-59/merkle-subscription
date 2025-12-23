use crate::error::SubscriptionError;
use crate::state::SubscriptionConfig;
use anchor_lang::prelude::*;

pub fn update_root(ctx: Context<UpdateRoot>, new_root: [u8; 32]) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.merkle_root = new_root;
    msg!("Merkle Root updated successfully.");
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateRoot<'info> {
    #[account(
        mut,
        has_one = authority @ SubscriptionError::Unauthorized,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, SubscriptionConfig>,
    pub authority: Signer<'info>,
}
