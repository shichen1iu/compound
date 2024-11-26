use crate::constants::*;
use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use mpl_core::accounts::{BaseAssetV1, BaseCollectionV1};
use mpl_core::instructions::BurnV1CpiBuilder;
use mpl_core::types::UpdateAuthority;
use mpl_core::ID as MPL_CORE_ID;

#[derive(Accounts)]
pub struct PermuteAsset<'info> {
    #[account(
        mut,
        constraint = permute_asset.owner == owner.key() @ CompoundError::InvalidAsset,
        constraint = permute_asset.update_authority == UpdateAuthority::Collection(permute_asset_collection.key()) @ CompoundError::UnknownAsset
    )]
    pub permute_asset: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub permute_asset_collection: Account<'info, BaseCollectionV1>,
    #[account(
        mut,
        seeds = [ REWARD_MINT_SEED ],
        bump
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [ STAKE_VAULT_SEED ],
        bump = stake_vault.bump
    )]
    pub stake_vault: Account<'info, StakeVault>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn process_permute_asset(
    ctx: Context<PermuteAsset>,
    permute_asset_total_currency: u32,
    create_time: i64,
) -> Result<()> {
    let permute_asset_collection = &ctx.accounts.permute_asset_collection;

    require_gt!(
        permute_asset_total_currency * 2 / 3,
        permute_asset_collection.current_size,
        CompoundError::PermuteAssetCurrentCurrencyTooHigh
    );

    let current_time = Clock::get()?.unix_timestamp;

    let time_since_create = current_time - create_time;
    require_gt!(
        time_since_create,
        30 * 24 * 60 * 60,
        CompoundError::PermuteAssetTooEarly
    );

    BurnV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.permute_asset.to_account_info())
        .collection(Some(
            &ctx.accounts.permute_asset_collection.to_account_info(),
        ))
        .payer(&ctx.accounts.owner.to_account_info())
        .authority(Some(&&ctx.accounts.owner.to_account_info()))
        .invoke()?;

    
    Ok(())
}
