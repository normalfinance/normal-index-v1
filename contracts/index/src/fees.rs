//! Time-based fee collection system for Normal Protocol
//! Implements 0.5% annual management fee with lazy calculation approach

use crate::events::{Events, IndexEvents};
use crate::storage::{
    get_accumulated_manager_fees, get_accumulated_protocol_fees, get_factory_safe,
    get_manager_fee_amount, get_minimum_shares_for_fee_collection, get_total_fees,
    set_accumulated_manager_fees, set_accumulated_protocol_fees, set_last_fee_collection,
    set_total_fees,
};
use normal_rust_types::UserFeeState;
use soroban_sdk::{contracttype, Address, Env, IntoVal, Symbol, Vec};
use utils::bump::bump_persistent;

#[derive(Clone)]
#[contracttype]
enum FeeDataKey {
    UserFeeState(Address),
    LastBatchCollection,
}

pub fn get_protocol_fee_amount_from_factory(e: &Env, user: &Address) -> u32 {
    match get_factory_safe(e) {
        Some(factory_address) => {
            match e.try_invoke_contract::<u32, soroban_sdk::Error>(
                &factory_address,
                &Symbol::new(e, "get_user_fee_rate"),
                Vec::from_array(e, [user.clone().into_val(e)]),
            ) {
                Ok(Ok(user_fee_rate)) => user_fee_rate,
                Ok(Err(_)) | Err(_) => e.invoke_contract::<u32>(
                    &factory_address,
                    &Symbol::new(e, "get_protocol_fee_amount"),
                    Vec::new(e),
                ),
            }
        }
        None => 0, // Return 0 if factory not set
    }
}

/// Get fee enabled status from Factory contract
/// Returns true if fees are enabled, defaults to true if factory not available (backwards compatibility)
pub fn get_fee_enabled_from_factory(e: &Env) -> bool {
    match get_factory_safe(e) {
        Some(factory_address) => {
            // Call Factory's get_index_fee_enabled() function
            e.invoke_contract::<bool>(
                &factory_address,
                &Symbol::new(e, "get_index_fee_enabled"),
                Vec::from_array(e, [e.current_contract_address().into_val(e)]),
            )
        }
        None => true, // Default to enabled if factory not available (backwards compatibility)
    }
}

/// Minimum fee threshold to avoid dust collection (in tokens)
const DEFAULT_MINIMUM_FEE_THRESHOLD: u128 = 1_000_000; // 1 token assuming 6 decimals

/// Seconds per year for precise calculations
const SECONDS_PER_YEAR: u64 = 31_536_000; // 365 * 24 * 60 * 60

/// Get user fee state, initializing if doesn't exist
pub fn get_user_fee_state(e: &Env, user: &Address) -> UserFeeState {
    let key = FeeDataKey::UserFeeState(user.clone());
    match e
        .storage()
        .persistent()
        .get::<FeeDataKey, UserFeeState>(&key)
    {
        Some(state) => {
            bump_persistent(e, &key);
            state
        }
        None => UserFeeState {
            balance: 0,
            last_fee_update: e.ledger().timestamp(),
            accrued_manager_fees: 0,
            accrued_protocol_fees: 0,
        },
    }
}

/// Write user fee state to storage
fn write_user_fee_state(e: &Env, user: &Address, state: &UserFeeState) {
    let key = FeeDataKey::UserFeeState(user.clone());
    e.storage().persistent().set(&key, state);
    bump_persistent(e, &key);
}

/// Get minimum fee threshold from Factory contract (universal threshold)
pub fn get_minimum_fee_threshold(e: &Env) -> u128 {
    match get_factory_safe(e) {
        Some(factory_address) => e.invoke_contract::<u128>(
            &factory_address,
            &Symbol::new(e, "get_minimum_fee_threshold"),
            Vec::new(e),
        ),
        None => DEFAULT_MINIMUM_FEE_THRESHOLD, // Fallback during testing/development
    }
}

// Note: Minimum fee threshold is now set universally by the Factory contract during deployment
// and cannot be changed thereafter. This ensures protocol-wide consistency.

/// Calculate time-based accrued fees for a user
/// Returns (manager_fee, protocol_fee)
pub fn calculate_accrued_fees(
    e: &Env,
    user: &Address,
    user_balance: i128,
    last_update_timestamp: u64,
    current_timestamp: u64,
) -> (u128, u128) {
    // Early return if fee collection is disabled for this index (query factory)
    if !get_fee_enabled_from_factory(e) {
        return (0, 0);
    }

    if current_timestamp <= last_update_timestamp || user_balance <= 0 {
        return (0, 0);
    }

    let time_elapsed = current_timestamp - last_update_timestamp;
    let user_balance_u128 = user_balance as u128;
    let time_elapsed_u128 = time_elapsed as u128;

    // Get separate fee rates
    let manager_fee_rate_bps = get_manager_fee_amount(e) as u128; // Manager fee (optional, max 2%)
    let protocol_fee_rate_bps = get_protocol_fee_amount_from_factory(e, user) as u128; // User-specific protocol fee based on tier

    // Calculate manager fee separately
    let manager_fee = if manager_fee_rate_bps > 0
        && user_balance_u128 > get_minimum_shares_for_fee_collection(e)
    {
        //Just calculate the flat fee for the time elapsed
        manager_fee_rate_bps * (time_elapsed_u128 / (SECONDS_PER_YEAR as u128))
    } else {
        0
    };

    // Calculate protocol fee separately
    let protocol_fee = if protocol_fee_rate_bps > 0
        && user_balance_u128 > get_minimum_shares_for_fee_collection(e)
    {
        //Just calculate the flat fee for the time elapsed
        protocol_fee_rate_bps * (time_elapsed_u128 / (SECONDS_PER_YEAR as u128))
    } else {
        0
    };

    (manager_fee, protocol_fee)
}

/// Collect accrued fees for a user if they meet the minimum threshold
/// Returns (manager_fee_collected, protocol_fee_collected)
pub fn collect_accrued_fees_if_any(e: &Env, user: &Address) -> (u128, u128) {
    let mut user_state = get_user_fee_state(e, user);
    let current_time = e.ledger().timestamp();

    // Calculate newly accrued fees since last update
    let (new_manager_fee, new_protocol_fee) = calculate_accrued_fees(
        e,
        user,
        user_state.balance,
        user_state.last_fee_update,
        current_time,
    );

    // Add to accumulated fees
    user_state.accrued_manager_fees += new_manager_fee;
    user_state.accrued_protocol_fees += new_protocol_fee;

    let minimum_threshold = get_minimum_fee_threshold(e);
    let total_accrued = user_state.accrued_manager_fees + user_state.accrued_protocol_fees;

    // Only collect if fees meet minimum threshold
    let (collected_manager, collected_protocol) = if total_accrued >= minimum_threshold {
        let manager_collected = user_state.accrued_manager_fees;
        let protocol_collected = user_state.accrued_protocol_fees;

        // Reset accumulated fees
        user_state.accrued_manager_fees = 0;
        user_state.accrued_protocol_fees = 0;

        // Update accumulated fees in storage
        if manager_collected > 0 || protocol_collected > 0 {
            let current_manager_fees = get_accumulated_manager_fees(e);
            let current_protocol_fees = get_accumulated_protocol_fees(e);
            let current_total_fees = get_total_fees(e);

            set_accumulated_manager_fees(e, &(current_manager_fees + manager_collected));
            set_accumulated_protocol_fees(e, &(current_protocol_fees + protocol_collected));
            set_total_fees(
                e,
                &(current_total_fees + manager_collected + protocol_collected),
            );
            set_last_fee_collection(e, &current_time);

            // Emit enhanced fee collection event
            let annual_fee_rate = get_manager_fee_amount(e); // This gets the annual fee amount
            Events::new(e).accrued_fees_collected(
                current_time,
                user.clone(),
                (user_state.balance as u128) + manager_collected + protocol_collected, // shares_before (approximation)
                user_state.balance as u128, // shares_after
                user_state.last_fee_update, // fee_period_start
                current_time,               // fee_period_end
                annual_fee_rate,
                manager_collected + protocol_collected, // total_fee_collected
                manager_collected,                      // manager_fee_portion
                protocol_collected,                     // protocol_fee_portion
            );

            // Also emit legacy event for backward compatibility
            Events::new(e).fee_collected(
                user.clone(),
                e.current_contract_address(),
                manager_collected + protocol_collected,
                manager_collected,
                protocol_collected,
            );
        }

        (manager_collected, protocol_collected)
    } else {
        (0, 0)
    };

    // Update timestamp regardless of whether fees were collected
    user_state.last_fee_update = current_time;

    // Write updated state back to storage
    write_user_fee_state(e, user, &user_state);

    (collected_manager, collected_protocol)
}

/// Initialize or update user balance for fee tracking (called during mints, transfers)
pub fn initialize_or_update_user_tracking(e: &Env, user: &Address, balance_change: i128) {
    let current_time = e.ledger().timestamp();

    // Get current state
    let mut user_state = get_user_fee_state(e, user);

    // Update balance
    user_state.balance = (user_state.balance + balance_change).max(0);
    user_state.last_fee_update = current_time;

    write_user_fee_state(e, user, &user_state);
}

/// Collect fees before user action (transfers, burns, etc.)
pub fn collect_fees_before_action(e: &Env, user: &Address, balance_change: i128) -> (u128, u128) {
    // First collect any accrued fees
    let (collected_manager, collected_protocol) = collect_accrued_fees_if_any(e, user);

    // Then update the balance after fee collection
    let mut user_state = get_user_fee_state(e, user);

    // Deduct collected fees from balance first, then apply the balance change
    let total_fees_collected = (collected_manager + collected_protocol) as i128;
    user_state.balance = user_state.balance.saturating_sub(total_fees_collected);
    user_state.balance = (user_state.balance + balance_change).max(0);
    user_state.last_fee_update = e.ledger().timestamp();

    write_user_fee_state(e, user, &user_state);

    (collected_manager, collected_protocol)
}

/// Get effective balance for a user (balance minus pending accrued fees)
pub fn get_effective_balance(e: &Env, user: &Address) -> i128 {
    let user_state = get_user_fee_state(e, user);
    let current_time = e.ledger().timestamp();

    // Calculate newly accrued fees
    let (new_manager_fee, new_protocol_fee) = calculate_accrued_fees(
        e,
        user,
        user_state.balance,
        user_state.last_fee_update,
        current_time,
    );

    let total_pending_fees = user_state.accrued_manager_fees
        + user_state.accrued_protocol_fees
        + new_manager_fee
        + new_protocol_fee;

    user_state
        .balance
        .saturating_sub(total_pending_fees as i128)
}

/// Preview fees that would be collected for a user (read-only)
pub fn preview_accrued_fees(e: &Env, user: &Address) -> (u128, u128) {
    let user_state = get_user_fee_state(e, user);
    let current_time = e.ledger().timestamp();

    let (new_manager_fee, new_protocol_fee) = calculate_accrued_fees(
        e,
        user,
        user_state.balance,
        user_state.last_fee_update,
        current_time,
    );

    (
        user_state.accrued_manager_fees + new_manager_fee,
        user_state.accrued_protocol_fees + new_protocol_fee,
    )
}

/// Batch collect fees from multiple users (for periodic collection)
/// Returns total fees collected (manager_fees, protocol_fees)
pub fn batch_collect_fees(e: &Env, users: Vec<Address>) -> (u128, u128) {
    let mut total_manager_fees = 0u128;
    let mut total_protocol_fees = 0u128;

    for user in users.iter() {
        let (manager_collected, protocol_collected) = collect_accrued_fees_if_any(e, &user);
        total_manager_fees += manager_collected;
        total_protocol_fees += protocol_collected;
    }

    (total_manager_fees, total_protocol_fees)
}

/// Get all users with significant accrued fees (for batch collection)
/// Returns users who have fees above the minimum threshold
pub fn get_users_with_accrued_fees(e: &Env, user_addresses: Vec<Address>) -> Vec<Address> {
    let minimum_threshold = get_minimum_fee_threshold(e);
    let mut users_with_fees = Vec::new(e);

    for user in user_addresses.iter() {
        let (manager_fees, protocol_fees) = preview_accrued_fees(e, &user);
        if manager_fees + protocol_fees >= minimum_threshold {
            users_with_fees.push_back(user);
        }
    }

    users_with_fees
}

/// Force collect fees from a user (admin emergency function)
pub fn force_collect_fees(e: &Env, user: &Address) -> (u128, u128) {
    let mut user_state = get_user_fee_state(e, user);
    let current_time = e.ledger().timestamp();

    // Calculate all accrued fees
    let (new_manager_fee, new_protocol_fee) = calculate_accrued_fees(
        e,
        user,
        user_state.balance,
        user_state.last_fee_update,
        current_time,
    );

    // Add to accumulated fees
    user_state.accrued_manager_fees += new_manager_fee;
    user_state.accrued_protocol_fees += new_protocol_fee;

    let manager_collected = user_state.accrued_manager_fees;
    let protocol_collected = user_state.accrued_protocol_fees;

    // Force collection regardless of threshold
    if manager_collected > 0 || protocol_collected > 0 {
        // Deduct collected fees from user balance
        user_state.balance = user_state
            .balance
            .saturating_sub((manager_collected + protocol_collected) as i128);

        // Reset accumulated fees
        user_state.accrued_manager_fees = 0;
        user_state.accrued_protocol_fees = 0;

        // Update accumulated fees in storage
        let current_manager_fees = get_accumulated_manager_fees(e);
        let current_protocol_fees = get_accumulated_protocol_fees(e);
        let current_total_fees = get_total_fees(e);

        set_accumulated_manager_fees(e, &(current_manager_fees + manager_collected));
        set_accumulated_protocol_fees(e, &(current_protocol_fees + protocol_collected));
        set_total_fees(
            e,
            &(current_total_fees + manager_collected + protocol_collected),
        );
        set_last_fee_collection(e, &current_time);

        // Emit enhanced fee collection event
        let annual_fee_rate = get_manager_fee_amount(e);
        let user_state = get_user_fee_state(e, user);
        Events::new(e).accrued_fees_collected(
            current_time,
            user.clone(),
            (user_state.balance as u128) + manager_collected + protocol_collected, // shares_before (approximation)
            user_state.balance as u128,                                            // shares_after
            user_state.last_fee_update, // fee_period_start
            current_time,               // fee_period_end
            annual_fee_rate,
            manager_collected + protocol_collected, // total_fee_collected
            manager_collected,                      // manager_fee_portion
            protocol_collected,                     // protocol_fee_portion
        );

        // Also emit legacy event for backward compatibility
        Events::new(e).fee_collected(
            user.clone(),
            e.current_contract_address(),
            manager_collected + protocol_collected,
            manager_collected,
            protocol_collected,
        );
    }

    // Update timestamp
    user_state.last_fee_update = current_time;
    write_user_fee_state(e, user, &user_state);

    (manager_collected, protocol_collected)
}

/// Get last batch collection timestamp
pub fn get_last_batch_collection(e: &Env) -> u64 {
    let key = FeeDataKey::LastBatchCollection;
    match e.storage().persistent().get::<FeeDataKey, u64>(&key) {
        Some(timestamp) => {
            bump_persistent(e, &key);
            timestamp
        }
        None => 0,
    }
}

/// Set last batch collection timestamp
pub fn set_last_batch_collection(e: &Env, timestamp: u64) {
    let key = FeeDataKey::LastBatchCollection;
    e.storage().persistent().set(&key, &timestamp);
    bump_persistent(e, &key);
}

// @dev from contract.rs

/// Called by token contract to enforce fee collection for transfers and burns
/// This ensures fees are collected regardless of where tokens are traded (external DEXes)
pub fn collect_fees_before_operation(
    e: Env,
    from: Address,
    amount: i128,
    to: Option<Address>, // Some(address) for transfers, None for burns
) -> (u128, u128) {
    // No auth required - this is called by the trusted token contract

    // Collect fees from sender before they transfer/burn tokens
    let (manager_fees, protocol_fees) = collect_fees_before_action(&e, &from, -amount);

    // Update tracking based on operation type
    match to {
        Some(recipient) => {
            // Transfer: update tracking for both users
            initialize_or_update_user_tracking(&e, &from, -amount); // Sender: reduce balance
            initialize_or_update_user_tracking(&e, &recipient, amount); // Receiver: increase balance
        }
        None => {
            // Burn: only update sender's tracking (tokens are destroyed)
            initialize_or_update_user_tracking(&e, &from, -amount);
        }
    }

    (manager_fees, protocol_fees)
}

/// Called by token contract during external mints to enforce fee collection
/// This prevents users from bypassing fees by acquiring tokens on external DEXes
pub fn collect_fees_before_mint(e: &Env, user: Address, amount: u128) -> (u128, u128) {
    // No auth required - this is called by the trusted token contract

    // Collect any accrued fees on user's existing balance
    let (manager_fees, protocol_fees) = collect_fees_before_action(e, &user, amount as i128);

    // Update user tracking with the new amount
    initialize_or_update_user_tracking(e, &user, amount as i128);

    (manager_fees, protocol_fees)
}
