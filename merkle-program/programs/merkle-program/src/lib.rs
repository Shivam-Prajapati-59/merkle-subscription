use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

pub use error::*;
#[allow(ambiguous_glob_reexports)]
pub use instructions::*;
pub use state::*;

declare_id!("FzA5z58t4RWgmDEEoHTrCnfXeZ4rgtnhU3P684J8rPdx");

#[program]
pub mod merkle_program {
    use super::*;

    /// Initialize the subscription config with an initial merkle root
    pub fn initialize(ctx: Context<Initialize>, initial_root: [u8; 32]) -> Result<()> {
        instructions::initialize(ctx, initial_root)
    }

    /// Update the merkle root (only authority can do this)
    pub fn update_root(ctx: Context<UpdateRoot>, new_root: [u8; 32]) -> Result<()> {
        instructions::update_root(ctx, new_root)
    }

    /// Verify a user's subscription using merkle proof
    pub fn verify_subscription(
        ctx: Context<VerifySubscription>,
        proof: Vec<[u8; 32]>,
        expiration: i64,
    ) -> Result<()> {
        instructions::verify_subscription(ctx, proof, expiration)
    }
}
