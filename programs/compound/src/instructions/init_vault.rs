use crate::constants::*;
use crate::state::StakeValut;
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
pub struct InitVault<'info> {
    #[account(
        init,
        payer = payer,
        space = StakeValut::LEN,
        seeds = [STAKE_VALUT_SEED],
        bump
    )]
    pub stake_valut: Account<'info, StakeValut>,
    #[account(
        init,
        payer = payer,
        seeds = [REWARD_MINT_SEED],
        bump,
        mint::decimals = 9,
        mint::authority = reward_mint,
        mint::freeze_authority = reward_mint,
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
    /// CHECK: this account will be checked by the mpl_core program
    pub compound_collection_update_authority: Option<UncheckedAccount<'info>>,
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
    create_reward_mint(&ctx)?;
    create_compound_collection(
        &ctx,
        &compound_collection_name,
        &compound_collection_uri,
        compound_collection_max_supply,
    )?;
    let stake_valut = &mut ctx.accounts.stake_valut;
    **stake_valut = StakeValut {
        reward_mint: ctx.accounts.reward_mint.key(),
        collection_a: ctx.accounts.collection_a.key(),
        collection_b: ctx.accounts.collection_b.key(),
        bump: ctx.bumps.stake_valut,
        compound_collection: ctx.accounts.compound_collection.key(),
        compound_asset_edition: 0,
        compound_collection_max_supply: compound_collection_max_supply,
    };

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
    let update_authority = match &ctx.accounts.compound_collection_update_authority {
        Some(update_authority) => Some(update_authority.to_account_info()),
        None => None,
    };

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

    CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .collection(&ctx.accounts.compound_collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .update_authority(update_authority.as_ref())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(compound_collection_name.to_string())
        .uri(compound_collection_uri.to_string())
        .plugins(compound_collection_plugins)
        .invoke()?;
    Ok(())
}