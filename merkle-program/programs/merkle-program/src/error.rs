use anchor_lang::prelude::*;
#[error_code]
pub enum SubscriptionError {
    #[msg("You are not authorized to update the root.")]
    Unauthorized,
    #[msg("Invalid Merkle proof provided.")]
    InvalidProof,
    #[msg("Your subscription has expired.")]
    SubscriptionExpired,
}
