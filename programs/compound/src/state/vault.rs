use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub bump: u8,
    pub pool_num: u8,
    pub reward_mint: Pubkey,
}
