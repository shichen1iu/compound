use crate::error::*;
use anchor_lang::prelude::*;

pub fn calculate_permute_amount(permute_asset_total_currency: u32) -> Result<u64> {
    let permute_amount: u64 = (3_000_000_000 as u64)
        .checked_mul(1_000 as u64)
        .ok_or(CompoundError::ArithmeticOverflow)?
        .checked_div(permute_asset_total_currency as u64)
        .ok_or(CompoundError::ArithmeticOverflow)?;

    Ok(permute_amount)
}
