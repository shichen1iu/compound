use crate::constants::*;
use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::{CreateV2CpiBuilder, TransferV1CpiBuilder},
    types::{Edition, Plugin, PluginAuthority, PluginAuthorityPair, UpdateAuthority},
    ID as MPL_CORE_ID,
};
#[derive(Accounts)]
pub struct StakeAsset<'info> {
    #[account(
        init,
        seeds = [STAKE_DETAILS_SEED, staker.key().as_ref()],
        bump,
        payer = staker,
        space = 8 + StakeDetails::INIT_SPACE,
    )]
    pub stake_details: Account<'info, StakeDetails>,
    #[account(
        mut,
        seeds = [STAKE_VAULT_SEED],
        bump = stake_vault.bump,
        has_one = collection_a,
        has_one = collection_b,
        has_one = compound_collection,
    )]
    pub stake_vault: Account<'info, StakeVault>,
    pub collection_a: Account<'info, BaseCollectionV1>,
    pub collection_b: Account<'info, BaseCollectionV1>,
    #[account(
        mut,
        constraint = asset_a.owner == staker.key() @ CompoundError::StakerAssetMismatch,
        constraint = asset_a.update_authority == UpdateAuthority::Collection(collection_a.key()) @ CompoundError::UnknownAsset
    )]
    pub asset_a: Account<'info, BaseAssetV1>,
    #[account(
        mut,
        constraint = asset_b.owner == staker.key() @ CompoundError::StakerAssetMismatch,
        constraint = asset_b.update_authority == UpdateAuthority::Collection(collection_b.key()) @ CompoundError::UnknownAsset
    )]
    pub asset_b: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub compound_collection: Account<'info, BaseCollectionV1>,
    #[account(mut)]
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

    let mut compound_asset_plugin: Vec<PluginAuthorityPair> = vec![];

    let edition_plugin = PluginAuthorityPair {
        plugin: Plugin::Edition(Edition {
            number: current_edition,
        }),
        authority: Some(PluginAuthority::UpdateAuthority),
    };

    let stake_valut_seeds: &[&[&[u8]]] = &[&[STAKE_VAULT_SEED, &[stake_vault.bump]]];

    compound_asset_plugin.push(edition_plugin);
    //将compound_asset mint 给staker
    CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.compound_asset.to_account_info())
        .payer(&&ctx.accounts.staker.to_account_info())
        .owner(Some(&ctx.accounts.staker.to_account_info()))
        .collection(Some(&ctx.accounts.compound_collection.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugins(compound_asset_plugin)
        .name(compound_asset_name)
        .uri(compound_asset_uri)
        .authority(Some(&ctx.accounts.stake_vault.to_account_info()))
        .invoke_signed(stake_valut_seeds)?;

    stake_details.bump = ctx.bumps.stake_details;
    stake_details.start_time = stake_start_time;
    stake_details.compound_asset = ctx.accounts.compound_asset.key();
    stake_details.asset_a = ctx.accounts.asset_a.key();
    stake_details.asset_b = ctx.accounts.asset_b.key();
    stake_details.asset_a_currency = collection_a.current_size;
    stake_details.asset_b_currency = collection_b.current_size;
    Ok(())
}
