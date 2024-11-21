use crate::constants::*;
use crate::error::*;
use crate::state::*;
use crate::utils::calculate_rewards;
use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{mpl_token_metadata::instructions::MintV1CpiBuilder, Metadata},
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::{BurnV1CpiBuilder, TransferV1CpiBuilder},
    ID as MPL_CORE_ID,
};

#[derive(Accounts)]
pub struct UnstakeAsset<'info> {
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
        seeds = [
            STAKE_DETAILS_SEED,
            staker.key().as_ref(),
            asset_a.key().as_ref(),
            asset_b.key().as_ref()
        ],
        bump = stake_details.bump,
        has_one = compound_asset,
        has_one = asset_a,
        has_one = asset_b,
    )]
    pub stake_details: Account<'info, StakeDetails>,
    #[account(
        mut,
        seeds = [REWARD_MINT_SEED],
        bump
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
    #[account(
        init_if_needed,
        payer = staker,
        associated_token::mint = reward_mint,
        associated_token::authority = staker,
        associated_token::token_program = token_program,
    )]
    pub reward_mint_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub compound_asset: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub compound_collection: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub asset_a: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub asset_b: Account<'info, BaseAssetV1>,
    #[account(mut)]
    pub collection_a: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub collection_b: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub staker: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    #[account(address = sysvar::instructions::id())]
    /// CHECK: this account is checked by token metadata
    pub sysvar_instructions: UncheckedAccount<'info>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn process_unstake_asset(ctx: Context<UnstakeAsset>) -> Result<()> {
    let stake_end_time = Clock::get()?.unix_timestamp;
    let stake_details = &mut ctx.accounts.stake_details;
    let stake_start_time = stake_details.start_time;

    let stake_time = stake_end_time - stake_start_time;

    require!(stake_details.is_staked, CompoundError::NotStaked);

    require_gt!(stake_time, MIN_STAKE_TIME, CompoundError::StakeTimeTooShort);

    let asset_a_currency = stake_details.asset_a_currency;
    let asset_b_currency = stake_details.asset_b_currency;

    let reward_amount =
        calculate_rewards(stake_time, asset_a_currency as u64, asset_b_currency as u64)?;

    let stake_vaults_seeds: &[&[&[u8]]] = &[&[STAKE_VAULT_SEED, &[ctx.accounts.stake_vault.bump]]];

    // 铸造奖励token给staker
    MintV1CpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
        .token(&ctx.accounts.reward_mint_ata.to_account_info())
        .metadata(&ctx.accounts.reward_mint_metadata.to_account_info())
        .mint(&ctx.accounts.reward_mint.to_account_info())
        .authority(&ctx.accounts.stake_vault.to_account_info())
        .payer(&ctx.accounts.staker.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
        .spl_token_program(&ctx.accounts.token_program.to_account_info())
        .spl_ata_program(&ctx.accounts.associated_token_program.to_account_info())
        .amount(reward_amount)
        .invoke_signed(stake_vaults_seeds)?;

    // 将nft a 转移给staker
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset_a.to_account_info())
        .payer(&ctx.accounts.staker.to_account_info())
        .authority(Some(&ctx.accounts.stake_vault.to_account_info()))
        .new_owner(&ctx.accounts.staker.to_account_info())
        .invoke_signed(stake_vaults_seeds)?;

    // 将nft b 转移给staker
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset_b.to_account_info())
        .payer(&ctx.accounts.staker.to_account_info())
        .authority(Some(&ctx.accounts.stake_vault.to_account_info()))
        .new_owner(&ctx.accounts.staker.to_account_info())
        .invoke_signed(stake_vaults_seeds)?;

    // 销毁compound_asset
    BurnV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.compound_asset.to_account_info())
        .collection(Some(&ctx.accounts.compound_collection.to_account_info()))
        .payer(&ctx.accounts.staker.to_account_info())
        .authority(Some(&&ctx.accounts.staker.to_account_info()))
        .invoke()?;

    let compound_asset_id = stake_details.compound_id;
    let stake_vault = &mut ctx.accounts.stake_vault;
    stake_vault.available_ids.push(compound_asset_id);

    Ok(())
}
