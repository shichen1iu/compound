use crate::constants::*;
use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::{CreateV2CpiBuilder, TransferV1CpiBuilder},
    types::{Edition, Plugin, PluginAuthority, PluginAuthorityPair, UpdateAuthority},
    ID as MPL_CORE_ID,
};
#[derive(Accounts)]
pub struct StakeAsset<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump = vault.bump,
        has_one = reward_mint,
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        init,
        seeds = [
            STAKE_DETAILS_SEED, 
            staker.key().as_ref(), 
            asset_a.key().as_ref(),
            asset_b.key().as_ref()
        ],
        bump,
        payer = staker,
        space = 8 + StakeDetails::INIT_SPACE,
    )]
    pub stake_details: Account<'info, StakeDetails>,
    #[account(
        mut,
        seeds = [COMPOUND_POOL_SEED],
        bump = compound_pool.bump,
        has_one = collection_a,
        has_one = collection_b,
        has_one = compound_collection,
    )]
    pub compound_pool: Account<'info, CompoundPool>,
    pub collection_a: Account<'info, BaseCollectionV1>,
    pub collection_b: Account<'info, BaseCollectionV1>,
    pub reward_mint: InterfaceAccount<'info, Mint>,
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
    let compound_pool = &mut ctx.accounts.compound_pool;
    let vault = &mut ctx.accounts.vault;

    //检查stake_vault的available_ids是否为空
    require!(
        !compound_pool.available_ids.is_empty(),
        CompoundError::NoAvailableIds
    );

    //将nft a 转移到stake_valut
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset_a.to_account_info())
        .collection(Some(&collection_a.to_account_info()))
        .payer(&ctx.accounts.staker.to_account_info())
        .authority(Some(&ctx.accounts.staker.to_account_info()))
        .new_owner(&vault.to_account_info())
        .invoke()?;

    //将nft b 转移到stake_valut
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset_b.to_account_info())
        .collection(Some(&collection_b.to_account_info()))
        .payer(&ctx.accounts.staker.to_account_info())
        .authority(Some(&ctx.accounts.staker.to_account_info()))
        .new_owner(&vault.to_account_info())
        .invoke()?;

    //从stake_vault的available_ids中取出当前的id
    let compound_asset_id = compound_pool.available_ids.pop().unwrap();

    // 根据当前的edition设置compound_asset的name
    let compound_asset_name = format!("{} #{}", compound_asset_name, compound_asset_id);

    let mut compound_asset_plugin: Vec<PluginAuthorityPair> = vec![];

    let edition_plugin = PluginAuthorityPair {
        plugin: Plugin::Edition(Edition {
            number: compound_asset_id as u32,
        }),
        authority: Some(PluginAuthority::UpdateAuthority),
    };

    let vault_seeds: &[&[&[u8]]] = &[&[VAULT_SEED, &[vault.bump]]];

    compound_asset_plugin.push(edition_plugin);
    //将compound_asset mint 给staker
    CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.compound_asset.to_account_info())
        .payer(&ctx.accounts.staker.to_account_info())
        .owner(Some(&ctx.accounts.staker.to_account_info()))
        .collection(Some(&ctx.accounts.compound_collection.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugins(compound_asset_plugin)
        .name(compound_asset_name)
        .uri(compound_asset_uri)
        .authority(Some(&ctx.accounts.vault.to_account_info()))
        .invoke_signed(vault_seeds)?;

    *ctx.accounts.stake_details = StakeDetails{
        bump: ctx.bumps.stake_details,
        start_time: stake_start_time,
        asset_a: ctx.accounts.asset_a.key(),
        asset_b: ctx.accounts.asset_b.key(),
        compound_asset_id: compound_asset_id,
        compound_asset: ctx.accounts.compound_asset.key(),
        is_staked: true,
    };
    
    Ok(())
}
