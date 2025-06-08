use anchor_lang::prelude::*;

#[account]
pub struct DonorAccount {
    pub stream: Pubkey,  // Parent stream
    pub donor: Pubkey,   // Contributor's wallet
    pub amount: u64,     // Total contributed
    pub refunded: bool,  // Track refund status
    pub bump: u8,        // PDA bump
}

impl Space for DonorAccount {
    const INIT_SPACE: usize = 8      // Discriminator
        + 32    // stream: Pubkey
        + 32    // donor: Pubkey
        + 8     // amount: u64
        + 1     // refunded: bool
        + 1;    // bump: u8
}
