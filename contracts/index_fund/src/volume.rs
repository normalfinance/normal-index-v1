use soroban_sdk::Env;
use utils::constant::THIRTY_DAY;

/// Returns the 30-day bucket index for a ledger timestamp.
///
/// # Arguments
/// - `ts` (`u64`): Ledger timestamp in seconds.
///
/// # Returns
/// - `u64`: Month bucket index (`ts / THIRTY_DAY`).
pub fn get_month_bucket(ts: u64) -> u64 {
    ts / THIRTY_DAY
}

/// Resolves protocol and manager fee rates (in bps) from configured volume tiers.
///
/// The highest tier whose `min_monthly_volume` is less than or equal to
/// `current_volume` wins.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `current_volume` (`u128`): User monthly volume used for tier matching.
///
/// # Returns
/// - `(u32, u32)`: `(protocol_fee_bps, manager_fee_bps)`.
pub fn get_volume_tier_fee_bps(e: &Env, current_volume: u128) -> (u32, u32) {
    let tiers = crate::storage::get_trade_fee_tiers(e);
    let mut protocol_fee_bps = 0u32;
    let mut manager_fee_bps = 0u32;

    for i in 0..tiers.len() {
        let tier = tiers.get_unchecked(i);
        if current_volume >= tier.min_monthly_volume && tier.protocol_fee_bps >= protocol_fee_bps {
            protocol_fee_bps = tier.protocol_fee_bps;
            manager_fee_bps = tier.manager_fee_bps;
        }
    }

    (protocol_fee_bps, manager_fee_bps)
}
