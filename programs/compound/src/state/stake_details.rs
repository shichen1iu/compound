use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeDetails {
    pub bump: u8,
    pub start_time: i64,
    pub asset_a: Pubkey,
    pub asset_b: Pubkey,
    pub compound_asset_id: u16,
    pub compound_asset: Pubkey,
    pub is_staked: bool,
}
