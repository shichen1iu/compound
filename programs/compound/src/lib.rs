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

    pub fn init_vault(
        ctx: Context<InitVault>,
        compound_collection_name: String,
        compound_collection_uri: String,
        compound_collection_max_supply: u32,
    ) -> Result<()> {
        process_init_vault(
            ctx,
            compound_collection_name,
            compound_collection_uri,
            compound_collection_max_supply,
        )
    }

    pub fn stake_asset(
        ctx: Context<StakeAsset>,
        compound_asset_name: String,
        compound_asset_uri: String,
        asset_a_total_currency: u32,
        asset_b_total_currency: u32,
    ) -> Result<()> {
        process_stake_asset(
            ctx,
            compound_asset_name,
            compound_asset_uri,
            asset_a_total_currency,
            asset_b_total_currency,
        )
    }

    pub fn unstake_asset(ctx: Context<UnstakeAsset>) -> Result<()> {
        process_unstake_asset(ctx)
    }
}
