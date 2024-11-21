use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeDetails {
    pub bump: u8,
    pub start_time: i64,
    pub end_time: i64,
    pub asset_a: Pubkey,
    pub asset_b: Pubkey,
    pub asset_a_currency: u32,
    pub asset_b_currency: u32,
    pub compound_id: u16,
    pub compound_collection: Pubkey,
    pub compound_asset: Pubkey,
    pub reward_amount: u64,
    pub stake_count: u16,
    pub is_staked: bool,
}
