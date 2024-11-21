use crate::constants::*;
use crate::error::*;

pub fn calculate_rewards(stake_time: i64) -> Result<u64, CompoundError> {
    let days = stake_time / (24 * 60 * 60);

    let mut multiply_factor: u64 = 100;

    const REWARD_MULTIPLIERS: [(i64, u64); 4] = [
        (14, 100),  // 14天内: 1x
        (30, 120),  // 15-30天: 1.2x
        (90, 150),  // 31-90天: 1.5x
        (180, 200), // 90天以上: 2x
    ];

    for (threshold_days, mult) in REWARD_MULTIPLIERS.into_iter() {
        if days > threshold_days {
            multiply_factor = mult;
        }
    }

    BASE_DAILY_REWARD
        .checked_mul(multiply_factor)
        .ok_or(CompoundError::ArithmeticOverflow)?
        .checked_mul(stake_time as u64)
        .ok_or(CompoundError::ArithmeticOverflow)?
        .checked_div(100)
        .ok_or(CompoundError::ArithmeticOverflow)?
        .checked_div(24 * 60 * 60)
        .ok_or(CompoundError::ArithmeticOverflow)
}
