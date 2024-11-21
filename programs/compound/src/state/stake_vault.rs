use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeVault {
    pub bump: u8,
    pub reward_mint: Pubkey,
    pub collection_a: Pubkey,
    pub collection_b: Pubkey,
    pub compound_collection: Pubkey,
    pub compound_collection_max_supply: u32,
    #[max_len(3000)]
    pub available_ids: Vec<u16>,
    pub base_daily_reward: u64,
}
