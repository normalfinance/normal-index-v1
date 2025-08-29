use crate::storage::{
    get_fee_tier_config_with_default, get_user_30_day_volume, get_user_tier_cache,
    set_user_tier_cache, UserTierData,
};
use soroban_sdk::{Address, Env, Map};

pub struct TierCalculator;

impl TierCalculator {
    pub fn get_user_fee_rate(env: &Env, user: &Address) -> u32 {
        let current_time = env.ledger().timestamp();

        if let Some(cached_data) = get_user_tier_cache(env, user) {
            return cached_data.current_fee_rate_bps;
        }

        /// if cache, the below code is not run
        let volume_usd = get_user_30_day_volume(env, user);
        let threshold = Self::determine_user_tier_threshold(env, volume_usd);
        let fee_rate = Self::get_fee_rate_for_threshold(env, threshold);

        let tier_data = UserTierData {
            current_tier_threshold: threshold,
            current_fee_rate_bps: fee_rate,
            total_30_day_volume: volume_usd,
            last_calculated: current_time,
            last_volume_update: current_time,
        };
        set_user_tier_cache(env, user, &tier_data);

        fee_rate
    }

    pub fn determine_user_tier_threshold(env: &Env, volume_usd: u128) -> u128 {
        let config = get_fee_tier_config_with_default(env);
        let tier_rates = config.tier_rates;

        let mut applicable_threshold = 0u128;

        for threshold in tier_rates.keys() {
            if volume_usd >= threshold && threshold >= applicable_threshold {
                applicable_threshold = threshold;
            }
        }

        applicable_threshold
    }

    pub fn get_user_tier_data(env: &Env, user: &Address) -> UserTierData {
        let current_time = env.ledger().timestamp();

        if let Some(cached_data) = get_user_tier_cache(env, user) {
            let cache_age = current_time.saturating_sub(cached_data.last_calculated);
            if cache_age < 3600 {
                return cached_data;
            }
        }

        let volume_usd = get_user_30_day_volume(env, user);
        let threshold = Self::determine_user_tier_threshold(env, volume_usd);
        let fee_rate = Self::get_fee_rate_for_threshold(env, threshold);

        let tier_data = UserTierData {
            current_tier_threshold: threshold,
            current_fee_rate_bps: fee_rate,
            total_30_day_volume: volume_usd,
            last_calculated: current_time,
            last_volume_update: current_time,
        };

        set_user_tier_cache(env, user, &tier_data);

        tier_data
    }

    pub fn get_fee_rate_for_threshold(env: &Env, threshold: u128) -> u32 {
        let config = get_fee_tier_config_with_default(env);
        let tier_rates = config.tier_rates;

        match tier_rates.get(threshold) {
            Some(rate) => rate,
            None => tier_rates.get(0u128).unwrap_or(10u32),
        }
    }

    pub fn validate_tier_config(tier_rates: &Map<u128, u32>) -> bool {
        if tier_rates.is_empty() {
            return false;
        }

        for rate in tier_rates.values() {
            if rate > 10000 {
                return false;
            }
        }

        if !tier_rates.contains_key(0u128) {
            return false;
        }

        true
    }

    pub fn invalidate_user_cache(env: &Env, user: &Address, new_volume_usd: u128) {
        if let Some(cached_data) = get_user_tier_cache(env, user) {
            let volume_change = if new_volume_usd > cached_data.total_30_day_volume {
                new_volume_usd - cached_data.total_30_day_volume
            } else {
                cached_data.total_30_day_volume - new_volume_usd
            };

            let change_threshold = cached_data.total_30_day_volume / 10;
            let absolute_threshold = 10_000_0000000u128;

            if volume_change > change_threshold || volume_change > absolute_threshold {
                crate::storage::invalidate_user_tier_cache(env, user);
            }
        }
    }
}
