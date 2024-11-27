use anchor_lang::{prelude::*, solana_program::system_instruction};
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
pub struct BuyAsset<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        mut,
        seeds = [SALE_LIST_SEED, buy_asset.key().as_ref()],
        bump = sale_list.bump,
        has_one = sale_asset_collection,
        has_one = current_owner
    )]
    pub sale_list: Account<'info, SaleListing>,
    #[account(
        mut,
        constraint = buy_asset.owner == buyer.key() @ CompoundError::StakerAssetMismatch,
        constraint = buy_asset.update_authority == UpdateAuthority::Collection(sale_asset_collection.key()) @ CompoundError::UnknownAsset
    )]
    pub buy_asset: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub sale_asset_collection: Account<'info, BaseCollectionV1>,
    /// CHECK: this account is checked by the has_one constraint
    pub current_owner: AccountInfo<'info>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyAsset<'info> {}

pub fn process_buy_asset(ctx: Context<BuyAsset>, amount: u64) -> Result<()> {
    require_eq!(
        amount,
        ctx.accounts.sale_list.current_price,
        CompoundError::InvalidPrice
    );

    let vault_seeds: &[&[&[u8]]] = &[&[VAULT_SEED, &[ctx.accounts.vault.bump]]];

    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.buy_asset.to_account_info())
        .payer(&ctx.accounts.buyer.to_account_info())
        .collection(Some(&ctx.accounts.sale_asset_collection.to_account_info()))
        .authority(Some(&ctx.accounts.vault.to_account_info()))
        .new_owner(&ctx.accounts.buyer.to_account_info())
        .invoke_signed(vault_seeds)?;

    let transfer_instruction = system_instruction::transfer(
        &ctx.accounts.buyer.key(),
        &ctx.accounts.sale_list.current_owner,
        amount,
    );

    anchor_lang::solana_program::program::invoke_signed(
        &transfer_instruction,
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.current_owner.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[],
    )?;

    Ok(())
}
