use anchor_lang::prelude::*;

#[error_code]
pub enum CompoundError {
    #[msg("Asset does not belong to the specified collection")]
    UnknownAsset,
    #[msg("Max supply reached")]
    MaxSupplyReached,
    #[msg("Asset does not belong to the staker")]
    StakerAssetMismatch,
    #[msg("Max supply too large , max is 3000")]
    MaxSupplyTooLarge,
    #[msg("No available ids")]
    NoAvailableIds,
    #[msg("Stake time should be greater than 7 days")]
    StakeTimeTooShort,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Not staked")]
    NotStaked,
    #[msg("Invalid asset")]
    InvalidAsset,
}
