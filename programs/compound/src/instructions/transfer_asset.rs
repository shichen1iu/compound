// use anchor_lang::prelude::*;
// use mpl_core::{
//     accounts::{BaseAssetV1, BaseCollectionV1},
//     instructions::TransferV1CpiBuilder,
// };

// #[derive(Accounts)]
// pub struct TransferAsset<'info> {
//     #[account(mut)]
//     pub transfer_asset: Account<'info, BaseAssetV1>,
//     #[account(mut)]
//     pub transfer_assset_collection: Account<'info, BaseCollectionV1>,
//     #[account(mut)]
//     pub asset_owner: Signer<'info>,
//     pub asset_receiver: AccountInfo<'info>,
//     #[account(address = MPL_CORE_ID)]
//     /// CHECK: this account is checked by the address constraint
//     pub mpl_core_program: UncheckedAccount<'info>,
// }

// pub fn process_transfer_asset(ctx: Context<TransferAsset>, amount: u64) -> Result<()> {
//     TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
//         .asset(&ctx.accounts.transfer_asset.to_account_info())
//         .payer(&ctx.accounts.asset_owner.to_account_info())
//         .collection(Some(&ctx.accounts.transfer_assset_collection.to_account_info()))
//         .authority(Some(&ctx.accounts.asset_owner.to_account_info()))
//         .new_owner(&ctx.accounts.asset_receiver.to_account_info())
//         .invoke_signed(vault_seeds)?;
//     Ok(())
// }
