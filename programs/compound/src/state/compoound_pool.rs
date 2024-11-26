use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct CompoundPool {
    pub bump: u8,
    pub collection_a: Pubkey,
    pub collection_b: Pubkey,
    pub compound_collection: Pubkey,
    pub compound_collection_max_supply: u32,
    #[max_len(3000)]
    pub available_ids: Vec<u16>,
}
