use crate::constants::*;
use crate::error::*;
use anchor_lang::prelude::*;

pub fn calculate_rewards(
    stake_time: i64,
    asset_a_currency: u64,
    asset_b_currency: u64,
) -> Result<u64> {
    let days = stake_time / (24 * 60 * 60);
    // 计算时间奖励系数
    let mut time_multiplier: u64 = 100;
    const REWARD_MULTIPLIERS: [(i64, u64); 4] = [
        (14, 100),  // 14天内: 1x
        (30, 120),  // 15-30天: 1.2x
        (90, 150),  // 31-90天: 1.5x
        (180, 200), // 90天以上: 2x
    ];

    for (threshold_days, mult) in REWARD_MULTIPLIERS.into_iter() {
        if days > threshold_days {
            time_multiplier = mult;
        }
    }

    // 计算流通量奖励系数
    let circulation_multiplier = if asset_a_currency >= asset_b_currency {
        // 如果 asset_a 流通量更大，使用 asset_b 的系数
        asset_a_currency
            .checked_div(asset_b_currency)
            .ok_or(CompoundError::ArithmeticOverflow)?
            .min(300) // 设置最大倍数上限为 3x
    } else {
        // 如果 asset_b 流通量更大，使用 asset_a 的系数
        asset_b_currency
            .checked_div(asset_a_currency)
            .ok_or(CompoundError::ArithmeticOverflow)?
            .min(300) // 设置最大倍数上限为 3x
    };

    // 计算最终奖励
    let daily_reward = BASE_DAILY_REWARD
        .checked_mul(time_multiplier)
        .ok_or(CompoundError::ArithmeticOverflow)?
        .checked_mul(circulation_multiplier)
        .ok_or(CompoundError::ArithmeticOverflow)?
        .checked_mul(stake_time as u64)
        .ok_or(CompoundError::ArithmeticOverflow)?
        .checked_div(100)
        .ok_or(CompoundError::ArithmeticOverflow)?
        .checked_div(24 * 60 * 60)
        .ok_or(CompoundError::ArithmeticOverflow)?;
    Ok(daily_reward)
}
