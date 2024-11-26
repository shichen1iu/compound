pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("4N2LDvF4R6idwYbZaRJSmVfoTSQGhBisBLY9kZJZ2x8H");

#[program]
pub mod compound {
    use super::*;

    pub fn init_vault(ctx: Context<InitVault>) -> Result<()> {
        process_init_vault(ctx)
    }

    pub fn init_compound_pool(
        ctx: Context<InitCompoundPool>,
        compound_collection_name: String,
        compound_collection_uri: String,
        compound_collection_currency: u16,
        collection_a_currency: u16,
        collection_b_currency: u16,
        stake_daily_reward_amount: u16,
    ) -> Result<()> {
        process_init_compound_pool(
            ctx,
            compound_collection_name,
            compound_collection_uri,
            compound_collection_currency,
            collection_a_currency,
            collection_b_currency,
            stake_daily_reward_amount,
        )
    }

    pub fn stake_asset(
        ctx: Context<StakeAsset>,
        compound_asset_name: String,
        compound_asset_uri: String,
    ) -> Result<()> {
        process_stake_asset(ctx, compound_asset_name, compound_asset_uri)
    }

    pub fn unstake_asset(ctx: Context<UnstakeAsset>) -> Result<()> {
        process_unstake_asset(ctx)
    }
}
