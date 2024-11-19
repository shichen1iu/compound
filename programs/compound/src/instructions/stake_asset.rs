use crate::constants::*;
use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use mpl_core::instructions::CreateV2CpiBuilder;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::TransferV1CpiBuilder,
    types::UpdateAuthority,
    ID as MPL_CORE_ID,
};
#[derive(Accounts)]
pub struct StakeAsset<'info> {
    #[account(
        init,
        seeds = [STAKE_DETAILS_SEED, staker.key().as_ref()],
        bump,
        payer = staker,
        space = StakeDetails::INIT_SPACE,
    )]
    pub stake_details: Account<'info, StakeDetails>,
    #[account(
        mut,
        seeds = [STAKE_VAULT_SEED],
        bump = stake_vault.bump,
        has_one = collection_a,
        has_one = collection_b,
    )]
    pub stake_vault: Account<'info, StakeVault>,
    pub collection_a: Account<'info, BaseCollectionV1>,
    pub collection_b: Account<'info, BaseCollectionV1>,
    pub asset_a: Account<'info, BaseAssetV1>,
    pub asset_b: Account<'info, BaseAssetV1>,
    pub compound_collection: Account<'info, BaseCollectionV1>,
    pub compound_asset: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn process_stake_asset(
    ctx: Context<StakeAsset>,
    compound_asset_name: String,
    compound_asset_uri: String,
) -> Result<()> {
    require!(
        ctx.accounts.asset_a.update_authority
            == UpdateAuthority::Collection(ctx.accounts.stake_vault.collection_a),
        CompoundError::UnknownAsset
    );

    require!(
        ctx.accounts.asset_b.update_authority
            == UpdateAuthority::Collection(ctx.accounts.stake_vault.collection_b),
        CompoundError::UnknownAsset
    );

    let stake_start_time = Clock::get()?.unix_timestamp;
    let collection_a = &ctx.accounts.collection_a;
    let collection_b = &ctx.accounts.collection_b;
    let stake_details = &mut ctx.accounts.stake_details;
    let stake_vault = &mut ctx.accounts.stake_vault;

    //将nft a 转移到stake_valut
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset_a.to_account_info())
        .collection(Some(&collection_a.to_account_info()))
        .payer(&ctx.accounts.staker.to_account_info())
        .authority(Some(&ctx.accounts.staker.to_account_info()))
        .new_owner(&stake_vault.to_account_info())
        .invoke()?;

    //将nft b 转移到stake_valut
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset_b.to_account_info())
        .collection(Some(&collection_b.to_account_info()))
        .payer(&ctx.accounts.staker.to_account_info())
        .authority(Some(&ctx.accounts.staker.to_account_info()))
        .new_owner(&stake_vault.to_account_info())
        .invoke()?;

    // 获取当前的asset的edition
    let current_edition = stake_vault.compound_asset_edition + 1;

    // 检查当前的edition是否超过了compound_collection的最大供应量
    require!(
        current_edition <= stake_vault.compound_collection_max_supply,
        CompoundError::MaxSupplyReached
    );

    // 根据当前的edition设置compound_asset的name
    let compound_asset_name = format!("{} #{}", compound_asset_name, current_edition);

    let stake_vault_signers_seeds: &[&[&[u8]]] = &[&[STAKE_VAULT_SEED, &[stake_vault.bump]]];

    //将compound_asset mint 给staker
    CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.compound_asset.to_account_info())
        .authority(Some(&ctx.accounts.stake_vault.to_account_info()))
        .payer(&ctx.accounts.stake_vault.to_account_info())
        .owner(Some(&ctx.accounts.staker.to_account_info()))
        .update_authority(Some(&ctx.accounts.compound_collection.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(compound_asset_name)
        .uri(compound_asset_uri)
        .invoke_signed(stake_vault_signers_seeds)?;

    stake_details.bump = ctx.bumps.stake_details;
    stake_details.start_time = stake_start_time;
    stake_details.asset_a_currency = collection_a.current_size;
    stake_details.asset_b_currency = collection_b.current_size;
    Ok(())
}
