pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("5jTvubXW96sdGSTPqiR7wsj75SZMckNfcihaBrmtuU4q");

#[program]
pub mod compound {
    use super::*;

    pub fn init_vault(ctx: Context<InitializeVault>) -> Result<()> {
        process_init_vault(ctx)
    }
}
