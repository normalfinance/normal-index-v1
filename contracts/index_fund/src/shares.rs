use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::Env;
use utils::validate;

pub fn shares_to_nav(e: &Env, n_shares: u128, total_shares: u128, current_nav: u128) -> u128 {
    validate!(
        e,
        n_shares <= total_shares,
        IndexFundError::InvalidSharesDetected
    );

    let amount = if total_shares > 0 {
        // Use round-to-nearest for fair withdrawal calculation
        // current_nav.safe_fixed_mul_round(e, n_shares, total_shares)
        current_nav
            .fixed_mul_floor(n_shares, total_shares)
            .unwrap_or(0)
    } else {
        0
    };

    amount
}

pub fn nav_amount_to_shares(e: &Env, amount: u128, total_shares: u128, current_nav: u128) -> u128 {
    let n_shares = if current_nav > 0 {
        // Use round-to-nearest for fair share calculation
        // amount.safe_fixed_mul_round(e, reserve.total_shares, reserve.balance)
        amount
            .fixed_mul_floor(total_shares, current_nav)
            .unwrap_or(amount)
    } else {
        // must be case that total_shares == 0 for nice result for user
        validate!(e, total_shares == 0, IndexFundError::InvalidSharesDetected);

        amount
    };

    n_shares
}
