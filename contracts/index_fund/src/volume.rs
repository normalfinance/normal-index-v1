use soroban_sdk::Env;
use utils::constant::THIRTY_DAY;

pub fn get_month_bucket(ts: u64) -> u64 {
    ts / THIRTY_DAY
}

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
