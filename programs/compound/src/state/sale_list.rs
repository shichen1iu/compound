use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct SaleListing {
    pub bump: u8,
    pub current_owner: Pubkey,
    pub current_price: u64,
    pub listed_at: i64,
    pub sale_asset: Pubkey,
    pub sale_asset_collection: Pubkey,
}
