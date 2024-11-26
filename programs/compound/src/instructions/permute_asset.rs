use crate::constants::*;
use crate::error::*;
use crate::state::*;
use crate::utils::calculate_permute_amount;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::mpl_token_metadata::instructions::MintV1CpiBuilder;
use anchor_spl::metadata::Metadata;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::Mint;
use anchor_spl::token_interface::TokenAccount;
use mpl_core::accounts::{BaseAssetV1, BaseCollectionV1};
use mpl_core::instructions::BurnV1CpiBuilder;
use mpl_core::types::UpdateAuthority;
use mpl_core::ID as MPL_CORE_ID;

#[derive(Accounts)]
pub struct PermuteAsset<'info> {
    #[account(
        mut,
        seeds = [STAKE_VAULT_SEED],
        bump = stake_vault.bump,
        has_one = collection_a,
        has_one = collection_b,
        has_one = compound_collection,
    )]
    pub stake_vault: Box<Account<'info, StakeVault>>,
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
    #[account(
        mut,
        seeds = [b"metadata",metadata_program.key().as_ref(), reward_mint.key().as_ref()],
        bump,
        seeds::program = metadata_program.key(),
    )]
    /// CHECK: this account is checked by the metadatatoken program
    pub reward_mint_metadata: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = reward_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub permute_mint_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = sysvar::instructions::id())]
    /// CHECK: this account is checked by token metadata
    pub sysvar_instructions: UncheckedAccount<'info>,
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

    let permute_amount = calculate_permute_amount(permute_asset_total_currency)?;

    let stake_vaults_seeds: &[&[&[u8]]] = [&STAKE_VAULT_SEED.as_ref(), &[ctx.bumps]];

    // 铸造奖励token给staker
    MintV1CpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
        .token(&ctx.accounts.permute_mint_ata.to_account_info())
        .metadata(&ctx.accounts.reward_mint_metadata.to_account_info())
        .mint(&ctx.accounts.reward_mint.to_account_info())
        .authority(&ctx.accounts.stake_vault.to_account_info())
        .payer(&ctx.accounts.owner.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
        .spl_token_program(&ctx.accounts.token_program.to_account_info())
        .spl_ata_program(&ctx.accounts.associated_token_program.to_account_info())
        .amount(permute_amount)
        .invoke_signed(stake_vaults_seeds)?;

    Ok(())
}
