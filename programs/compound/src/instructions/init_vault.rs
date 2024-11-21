use crate::constants::*;
use crate::error::*;
use crate::state::StakeVault;
use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    metadata::{
        mpl_token_metadata::{instructions::CreateV1CpiBuilder, types::TokenStandard},
        Metadata,
    },
    token_2022::Token2022,
    token_interface::Mint,
};
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
pub struct InitVault<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + StakeVault::INIT_SPACE,
        seeds = [STAKE_VAULT_SEED],
        bump
    )]
    pub stake_vault: Box<Account<'info, StakeVault>>,
    #[account(
        init,
        payer = payer,
        seeds = [REWARD_MINT_SEED],
        bump,
        mint::decimals = 9,
        mint::authority = stake_vault,
        mint::freeze_authority = stake_vault,
        mint::token_program = token_program,
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"metadata",metadata_program.key().as_ref(), reward_mint.key().as_ref()],
        bump,
        seeds::program = metadata_program.key(),
    )]
    /// CHECK: this account is checked by the metadatatoken program
    pub reward_mint_metadata: UncheckedAccount<'info>,
    pub collection_a: Account<'info, BaseCollectionV1>,
    pub collection_b: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub compound_collection: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub metadata_program: Program<'info, Metadata>,
    #[account(address = sysvar::instructions::id())]
    /// CHECK: Instruction ssysvar account
    pub sysvar_instructions: UncheckedAccount<'info>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn process_init_vault(
    ctx: Context<InitVault>,
    compound_collection_name: String,
    compound_collection_uri: String,
    compound_collection_max_supply: u32,
) -> Result<()> {
    require_gt!(
        3000,
        compound_collection_max_supply,
        CompoundError::MaxSupplyTooLarge
    );
    create_reward_mint(&ctx)?;
    create_compound_collection(
        &ctx,
        &compound_collection_name,
        &compound_collection_uri,
        compound_collection_max_supply,
    )?;

    let stake_vault = &mut ctx.accounts.stake_vault;

    stake_vault.bump = ctx.bumps.stake_vault;
    stake_vault.reward_mint = ctx.accounts.reward_mint.key();
    stake_vault.collection_a = ctx.accounts.collection_a.key();
    stake_vault.collection_b = ctx.accounts.collection_b.key();
    stake_vault.compound_collection = ctx.accounts.compound_collection.key();
    stake_vault.compound_collection_max_supply = compound_collection_max_supply;
    // 使用 rev() 从max_supply到1小插入
    for i in (1..=compound_collection_max_supply).rev() {
        stake_vault.available_ids.push(i as u16);
    }

    Ok(())
}

fn create_reward_mint(ctx: &Context<InitVault>) -> Result<()> {
    let reward_mint_seed: &[&[&[u8]]] = &[&[REWARD_MINT_SEED, &[ctx.bumps.reward_mint]]];

    CreateV1CpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
        .metadata(&ctx.accounts.reward_mint_metadata.to_account_info())
        .mint(&ctx.accounts.reward_mint.to_account_info(), false)
        .authority(&ctx.accounts.reward_mint.to_account_info())
        .payer(&&ctx.accounts.payer.to_account_info())
        .update_authority(&ctx.accounts.reward_mint.to_account_info(), true)
        .spl_token_program(Some(&ctx.accounts.token_program.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
        .token_standard(TokenStandard::Fungible)
        .name(String::from("Compound Go"))
        .symbol(String::from("CPG"))
        .seller_fee_basis_points(0)
        .is_mutable(true)
        .uri("https://gray-managing-penguin-864.mypinata.cloud/ipfs/QmZeZtp39Nv4z4CP4fjvZLgH6wB4kULrv8ytxRcqc8rSJa".to_string())
        .invoke_signed(reward_mint_seed)?;
    Ok(())
}

fn create_compound_collection(
    ctx: &Context<InitVault>,
    compound_collection_name: &str,
    compound_collection_uri: &str,
    compound_collection_max_supply: u32,
) -> Result<()> {
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
            max_supply: Some(compound_collection_max_supply),
            name: Some(compound_collection_name.to_string()),
            uri: Some(compound_collection_uri.to_string()),
        }),
        authority: Some(PluginAuthority::UpdateAuthority),
    };
    compound_collection_plugins.push(master_edition_plugin);

    let stake_vault_signers_seeds: &[&[&[u8]]] = &[&[STAKE_VAULT_SEED, &[ctx.bumps.stake_vault]]];

    CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .collection(&ctx.accounts.compound_collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .update_authority(Some(&ctx.accounts.stake_vault.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(compound_collection_name.to_string())
        .uri(compound_collection_uri.to_string())
        .plugins(compound_collection_plugins)
        .invoke_signed(stake_vault_signers_seeds)?;

    
    Ok(())
}
