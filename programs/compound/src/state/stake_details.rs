use anchor_lang::prelude::*;

#[account]
pub struct StakeDetails {
    pub bump: u8,
    pub start_time: i64,
    pub end_time: i64,
    pub staker: Pubkey,
    pub asset_a: Pubkey,
    pub asset_b: Pubkey,
    pub collection_a: Pubkey,
    pub collection_b: Pubkey,
    pub asset_a_currency: u16,
    pub asset_b_currency: u16,
    pub reward_mint: Pubkey,
    pub compound_collection: Pubkey,
    pub compound_asset: Pubkey,
    pub reward_amount: u64,
}

impl StakeDetails {
    pub const LEN: usize = 8 + 8 + 8 + 32 + 32 + 32 + 32 + 2 + 2 + 32 + 32 + 8;
}
