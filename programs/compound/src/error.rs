use anchor_lang::prelude::*;

#[error_code]
pub enum CompoundError {
    #[msg("Asset does not belong to the specified collection")]
    UnknownAsset,
    #[msg("Max supply reached")]
    MaxSupplyReached,
    #[msg("Asset does not belong to the staker")]
    StakerAssetMismatch,
}
