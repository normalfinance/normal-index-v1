use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{panic_with_error, Address, Env, Vec};
use types::adapter::AdapterTradeParams;
use utils::validate;

use crate::errors::IndexFundError;

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
    let nav = get_current_nav(e);

    if total_shares == 0 || nav == 0 {
        return crate::storage::get_initial_price(e);
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
                crate::oracle::OracleUtils::get_token_price_usd(e, &component_address);

            // Calculate value: balance * price
            let component_value = balance.saturating_mul(token_price);
            total_value = total_value.saturating_add(component_value);
        }
    }

    total_value
}

pub fn execute_weight_based_mint(e: &Env, deposited_token: Address, deposited_amount: u128) {
    // Get all current components and their weights
    let components = crate::storage::get_all_components(e);

    if components.len() == 0 {
        // No components defined, just hold the deposited token as-is
        panic_with_error!(&e, IndexFundError::ComponentNotFound);
    }

    let mut swaps = Vec::new(e);

    // Get component addresses for iteration
    let component_addresses = crate::storage::get_component_registry(e);

    // For each component, calculate how much of the deposited amount should be allocated
    let len = component_addresses.len();
    for i in 0..len {
        let component_token = component_addresses.get_unchecked(i);
        let component = components.get(component_token.clone()).unwrap();

        // Calculate target amount based on weight (weight is in basis points, 10000 = 100%)
        let target_amount = (deposited_amount * component.weight) / 10000;

        if target_amount > 0 {
            if component_token == deposited_token {
                // No swap needed - the deposited token matches this component
                // Just update the component balance directly
                let current_balance =
                    crate::storage::get_component_balance_safe(e, component_token.clone())
                        .unwrap_or(0);
                crate::storage::set_component_balance(
                    e,
                    component_token.clone(),
                    current_balance + target_amount,
                );
            } else {
                // Need to swap deposited token for component token
                let swap = AdapterTradeParams {
                    token_in: deposited_token.clone(),
                    token_out: component_token.clone(),
                    amount_in: target_amount,
                    amount_out_min: (target_amount * 95) / 100, // 5% slippage tolerance
                    to: e.current_contract_address(),
                    asset: component.asset.clone(),
                    metadata: None,
                };

                swaps.push_back(swap);
            }
        }
    }

    // Execute all swaps if any are needed
    if swaps.len() > 0 {
        let swap_results = crate::index::execute_swaps(e, swaps);

        // Update component balances based on swap results
        let mut swap_index = 0;
        let len2 = component_addresses.len();
        for i in 0..len2 {
            let component_token = component_addresses.get_unchecked(i);
            let component = components.get(component_token.clone()).unwrap();
            let target_amount = (deposited_amount * component.weight) / 10000;

            if target_amount > 0 && component_token != deposited_token {
                // This component required a swap
                if swap_index < swap_results.len() {
                    let amount_received = swap_results.get(swap_index).unwrap_or(0u128);
                    let current_balance =
                        crate::storage::get_component_balance_safe(e, component_token.clone())
                            .unwrap_or(0);
                    crate::storage::set_component_balance(
                        e,
                        component_token.clone(),
                        current_balance + amount_received,
                    );
                    swap_index += 1;
                }
            }
        }
    }
}
