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
        stake_daily_reward_amount: u64,
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

    pub fn stake_asset(ctx: Context<StakeAsset>) -> Result<()> {
        process_stake_asset(ctx)
    }

    pub fn unstake_asset(ctx: Context<UnstakeAsset>) -> Result<()> {
        process_unstake_asset(ctx)
    }

    pub fn permute_asset(
        ctx: Context<PermuteAsset>,
        permute_asset_total_currency: u32,
        create_time: i64,
    ) -> Result<()> {
        process_permute_asset(ctx, permute_asset_total_currency, create_time)
    }

    pub fn sale_asset(ctx: Context<SaleAsset>, price: u64) -> Result<()> {
        process_sale_asset(ctx, price)
    }
}
