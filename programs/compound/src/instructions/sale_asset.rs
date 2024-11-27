use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::TransferV1CpiBuilder,
    types::UpdateAuthority,
    ID as MPL_CORE_ID,
};

use crate::{
    constants::{SALE_LIST_SEED, VAULT_SEED},
    error::CompoundError,
    state::{SaleListing, Vault},
};

#[derive(Accounts)]
pub struct SaleAsset<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, sale_asset.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        init,
        space = 8 + SaleListing::INIT_SPACE,
        payer = owner,
        seeds = [SALE_LIST_SEED],
        bump,
    )]
    pub sale_list: Account<'info, SaleListing>,
    #[account(
        mut,
        constraint = sale_asset.owner == owner.key() @ CompoundError::StakerAssetMismatch,
        constraint = sale_asset.update_authority == UpdateAuthority::Collection(sale_asset_collection.key()) @ CompoundError::UnknownAsset
    )]
    pub sale_asset: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub sale_asset_collection: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn process_sale_asset(ctx: Context<SaleAsset>, price: u64) -> Result<()> {
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.sale_asset.to_account_info())
        .payer(&ctx.accounts.owner.to_account_info())
        .collection(Some(&ctx.accounts.sale_asset_collection.to_account_info()))
        .authority(Some(&&ctx.accounts.owner.to_account_info()))
        .new_owner(&ctx.accounts.vault.to_account_info())
        .invoke()?;

    *ctx.accounts.sale_list = SaleListing {
        sale_asset: ctx.accounts.sale_asset.key(),
        sale_asset_collection: ctx.accounts.sale_asset_collection.key(),
        current_owner: ctx.accounts.owner.key(),
        current_price: price,
        listed_at: Clock::get()?.unix_timestamp,
    };
    Ok(())
}
