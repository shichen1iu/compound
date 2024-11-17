use anchor_lang::prelude::*;

#[account]
pub struct StakeValut {
    pub bump: u8,
    pub reward_mint: Pubkey,
    pub collection_a: Pubkey,
    pub collection_b: Pubkey,
}

impl StakeValut {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 32;
}
