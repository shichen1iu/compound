use crate::constants::*;
use crate::state::*;
use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::{
    metadata::{
        mpl_token_metadata::{instructions::CreateV1CpiBuilder, types::TokenStandard},
        Metadata,
    },
    token_2022::Token2022,
    token_interface::Mint,
};
#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Vault::INIT_SPACE,
        seeds = [VAULT_SEED],
        bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        init,
        payer = payer,
        seeds = [REWARD_MINT_SEED],
        bump,
        mint::decimals = 9,
        mint::authority = vault,
        mint::freeze_authority = vault,
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
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
    pub metadata_program: Program<'info, Metadata>,
    #[account(address = sysvar::instructions::id())]
    /// CHECK: Instruction ssysvar account
    pub sysvar_instructions: UncheckedAccount<'info>,
}

pub fn process_init_vault(ctx: Context<InitVault>) -> Result<()> {
    let vault_seeds: &[&[&[u8]]] = &[&[VAULT_SEED, &[ctx.bumps.vault]]];

    CreateV1CpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
        .metadata(&ctx.accounts.reward_mint_metadata.to_account_info())
        .mint(&ctx.accounts.reward_mint.to_account_info(), false)
        .authority(&ctx.accounts.vault.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .update_authority(&ctx.accounts.vault.to_account_info(), true)
        .spl_token_program(Some(&ctx.accounts.token_program.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
        .token_standard(TokenStandard::Fungible)
        .name(String::from("Compound Go"))
        .symbol(String::from("CPG"))
        .seller_fee_basis_points(0)
        .is_mutable(true)
        .uri("https://gray-managing-penguin-864.mypinata.cloud/ipfs/QmZeZtp39Nv4z4CP4fjvZLgH6wB4kULrv8ytxRcqc8rSJa".to_string())
        .invoke_signed(vault_seeds)?;

    *ctx.accounts.vault = Vault {
        bump: ctx.bumps.vault,
        pool_num: 0,
        reward_mint: ctx.accounts.reward_mint.key(),
    };
    Ok(())
}
