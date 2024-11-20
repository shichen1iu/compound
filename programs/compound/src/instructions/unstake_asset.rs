use crate::constants::*;
use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use mpl_core::accounts::BaseAssetV1;

#[derive(Accounts)]
pub struct UnstakeAsset<'info> {
    #[account(
        mut,
        seeds = [STAKE_DETAILS_SEED],
        bump = stake_detail.bump,
        has_one = compound_asset,
        has_one = asset_a,
        has_one = asset_b,
    )]
    pub stake_detail: Account<'info, StakeDetails>,
    #[account(
        mut,
        seeds = [REWARD_MINT_SEED],
        bump
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub compound_asset: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub asset_a: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub asset_b: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub staker: Signer<'info>,
}

pub fn process_unstake_asset(ctx: Context<UnstakeAsset>) -> Result<()> {
    let stake_end_time = Clock::get()?.unix_timestamp;

    
    Ok(())
}
