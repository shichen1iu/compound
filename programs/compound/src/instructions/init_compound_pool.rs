use crate::constants::*;
use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseCollectionV1,
    instructions::CreateCollectionV2CpiBuilder,
    types::{
        Creator, MasterEdition, Plugin, PluginAuthority, PluginAuthorityPair, Royalties, RuleSet,
    },
    ID as MPL_CORE_ID,
};
#[derive(Accounts)]
#[instruction(compound_collection_name: String)]
pub struct InitCompoundPool<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        init,
        payer = payer,
        space = 8 + CompoundPool::INIT_SPACE,
        seeds = [COMPOUND_POOL_SEED],
        bump
    )]
    pub compound_pool: Box<Account<'info, CompoundPool>>,
    pub collection_a: Account<'info, BaseCollectionV1>,
    pub collection_b: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub compound_collection: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn process_init_compound_pool(
    ctx: Context<InitCompoundPool>,
    compound_collection_name: String,
    compound_collection_uri: String,
    compound_collection_currency: u16,
    collection_a_currency: u16,
    collection_b_currency: u16,
    stake_daily_reward_amount: u64,
) -> Result<()> {
    //compound_collection_max_supply 不能大于3000
    require_gt!(
        3000,
        compound_collection_currency,
        CompoundError::MaxSupplyTooLarge
    );

    let mut compound_collection_plugins: Vec<PluginAuthorityPair> = vec![];

    //添加版权插件
    let royalties_plugin = PluginAuthorityPair {
        plugin: Plugin::Royalties(Royalties {
            basis_points: 500,
            creators: vec![Creator {
                address: ctx.accounts.payer.key(),
                percentage: 100,
            }],
            rule_set: RuleSet::None,
        }),
        authority: Some(PluginAuthority::UpdateAuthority),
    };
    compound_collection_plugins.push(royalties_plugin);

    //添加Master Edition插件
    let master_edition_plugin = PluginAuthorityPair {
        plugin: Plugin::MasterEdition(MasterEdition {
            max_supply: Some(compound_collection_currency as u32),
            name: Some(compound_collection_name.to_string()),
            uri: Some(compound_collection_uri.to_string()),
        }),
        authority: Some(PluginAuthority::UpdateAuthority),
    };
    compound_collection_plugins.push(master_edition_plugin);

    let vault_signers_seeds: &[&[&[u8]]] = &[&[VAULT_SEED, &[ctx.accounts.vault.bump]]];

    CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .collection(&ctx.accounts.compound_collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .update_authority(Some(&ctx.accounts.vault.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(compound_collection_name.to_string())
        .uri(compound_collection_uri.to_string())
        .plugins(compound_collection_plugins)
        .invoke_signed(vault_signers_seeds)?;

    let vault = &mut ctx.accounts.vault;

    let mut ids: Vec<u16> = vec![];
    // 使用 rev() 从max_supply到1小插入
    ids.extend((1..=compound_collection_currency).rev().map(|i| i as u16));

    **ctx.accounts.compound_pool = CompoundPool {
        bump: ctx.bumps.compound_pool,
        collection_a: ctx.accounts.collection_a.key(),
        collection_b: ctx.accounts.collection_b.key(),
        compound_collection: ctx.accounts.compound_collection.key(),
        compound_collection_currency: compound_collection_currency,
        compound_asset_name: compound_collection_name,
        compound_asset_uri: compound_collection_uri,
        collection_a_currency: collection_a_currency,
        collection_b_currency: collection_b_currency,
        stake_daily_reward_amount: stake_daily_reward_amount,
        available_ids: ids,
    };

    vault.pool_num += 1;

    Ok(())
}
