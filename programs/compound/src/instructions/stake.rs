pub fn calculate_rewards(days_staked: u64) -> u64 {
    let base_reward = BASE_DAILY_REWARD;
    let mut multiplier = 100; // 基础倍数

    for (threshold, mult) in REWARD_MULTIPLIERS.iter() {
        if days_staked >= *threshold {
            multiplier = *mult;
        }
    }

    base_reward
        .checked_mul(days_staked)
        .unwrap()
        .checked_mul(multiplier as u64)
        .unwrap()
        .checked_div(100)
        .unwrap()
} 