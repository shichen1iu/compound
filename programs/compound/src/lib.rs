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

    pub fn init_vault(
        ctx: Context<InitVault>,
        compound_collection_name: String,
        compound_collection_uri: String,
        compound_collection_max_supply: u32,
    ) -> Result<()> {
        process_init_vault(
            ctx,
            compound_collection_name,
            compound_collection_uri,
            compound_collection_max_supply,
        )
    }
}
