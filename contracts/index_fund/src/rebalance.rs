use soroban_sdk::{panic_with_error, Address, Env};
use types::component::RebalanceParams;

use crate::errors::IndexFundError;

pub fn can_rebalance(e: &Env) -> bool {
    let current_time = e.ledger().timestamp();
    let last_rebalance = crate::storage::get_last_rebalance_ts(&e);
    let threshold = crate::storage::get_rebalance_threshold(&e);

    current_time >= last_rebalance + threshold
}

// Rebalancing helper functions
pub fn validate_rebalance(e: &Env, caller: &Address) {
    let access_control = AccessControl::new(e);
    let is_rebalance_authority = access_control
        .get_role_address_status_safe(&Role::RebalanceAuthorities, caller)
        .unwrap_or(false);

    // Allow admin or rebalance authority
    if !access_control.address_has_role(caller, &Role::Admin) && !is_rebalance_authority {
        panic_with_error!(e, IndexFundError::UnauthorizedRebalance);
        s
    }
}

pub fn rebalance(e: &Env, admin: Address, params: RebalanceParams, nav_before: u128) {
    let start_time = e.ledger().timestamp();

    let can_rebalance = can_rebalance(e.clone());
    if !can_rebalance {
        panic_with_error!(e, IndexFundError::RebalanceNotAllowed);
    }

    // Generate and execute swap transactions to align current balances with target weights
    let swaps = crate::index::generate_rebalance_swaps(e, &params);
    let total_swaps = swaps.len() as u32;

    if total_swaps > 0 {
        let _swap_results = crate::index::execute_swaps(e, swaps);
    }

    // Capture end state for enhanced event
    let end_time = e.ledger().timestamp();
    let nav_after = Self::get_current_nav(e.clone()) as u128;
    let duration_ms = (end_time - start_time) * 1000; // Convert to milliseconds
    let performance_delta = (nav_after as i128) - (nav_before as i128);

    // Emit enhanced completion event (no components updated, only swaps)
    Events::new(e).rebalance_completed(
        end_time,
        admin,
        0, // components_updated: 0 since rebalancing doesn't update components anymore
        total_swaps,
        performance_delta,
        nav_before,
        nav_after,
        duration_ms,
    );
}
