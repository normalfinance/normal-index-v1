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

pub fn get_current_share_price(e: &Env) -> u128 {
    let total_shares = token_share::get_total_shares(e);
    if total_shares == 0 {
        let ip = crate::storage::get_initial_price(e);
        return if ip < 0 { 0 } else { ip as u128 };
    }

    let nav = get_current_nav(e);
    if nav == 0 {
        let ip = crate::storage::get_initial_price(e);
        return if ip < 0 { 0 } else { ip as u128 };
    }

    // Share price = Total Portfolio Value / Total Shares
    nav / total_shares
}

pub fn get_current_nav(e: &Env) -> u128 {
    let mut total_value: u128 = 0;

    // Get all component addresses from registry
    let component_addresses = crate::storage::get_component_registry(e);

    // Iterate through each component to calculate total portfolio value
    let len = component_addresses.len();
    for i in 0..len {
        let component_address = component_addresses.get_unchecked(i);
        // Get the component balance (how much of this token the index holds)
        let balance = match crate::storage::get_component_balance_safe(e, component_address.clone())
        {
            Some(bal) => bal,
            None => 0u128, // If no balance stored, treat as 0
        };

        if balance > 0 {
            // Get the token price - for now we'll use a placeholder approach
            let token_price =
                crate::oracle::OracleUtils::get_token_price_usd(e, component_address.clone());

            // Calculate value: balance * price
            let component_value = balance.saturating_mul(token_price);
            total_value = total_value.saturating_add(component_value);
        }
    }

    total_value
}
