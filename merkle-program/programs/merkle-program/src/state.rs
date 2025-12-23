use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct SubscriptionConfig {
    pub authority: Pubkey,     // Your backend's public key
    pub merkle_root: [u8; 32], // The only data that changes
    pub bump: u8,              // PDA bump seed
}
