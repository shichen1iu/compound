use anchor_lang::prelude::*;

#[account]
pub struct StakeValut {
    pub bump: u8,
    pub reward_mint: Pubkey,
    pub collection_a: Pubkey,
    pub collection_b: Pubkey,
    pub compound_collection: Pubkey,
    pub compound_asset_edition: u32,
    pub compound_collection_max_supply: u32,
}

impl StakeValut {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 32 + 32 + 4 + 4;
}
