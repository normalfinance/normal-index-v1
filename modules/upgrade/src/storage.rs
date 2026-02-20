use soroban_sdk::{contracttype, BytesN, Env};
use utils::bump::bump_instance;

/// Instance storage keys for upgrade staging state.
#[derive(Clone)]
#[contracttype]
enum DataKey {
    /// Ledger timestamp after which a committed upgrade may be applied.
    UpgradeDeadline,
    /// WASM hash staged for a future upgrade.
    FutureWASM,
}

/// Returns the currently configured upgrade deadline timestamp.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
///
/// # Returns
/// - `u64`: Upgrade deadline timestamp, or `0` when unset.
pub fn get_upgrade_deadline(e: &Env) -> u64 {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::UpgradeDeadline)
        .unwrap_or(0)
}

/// Sets the upgrade deadline timestamp.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `value` (`&u64`): New upgrade deadline timestamp.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn put_upgrade_deadline(e: &Env, value: &u64) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::UpgradeDeadline, value);
}

/// Returns the staged future WASM hash, if any.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
///
/// # Returns
/// - `Option<BytesN<32>>`: Staged upgrade hash if present.
pub fn get_future_wasm(e: &Env) -> Option<BytesN<32>> {
    bump_instance(e);
    e.storage().instance().get(&DataKey::FutureWASM)
}

/// Stores a future WASM hash to be applied during upgrade.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `value` (`&BytesN<32>`): Future WASM hash.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn put_future_wasm(e: &Env, value: &BytesN<32>) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::FutureWASM, value);
}
