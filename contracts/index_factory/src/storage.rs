use normal_rust_types::StorageError;
use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, Map, String, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

#[derive(Clone)]
#[contracttype]
enum DataKey {
    SwapUtility, // DEX Aggregator

    OracleRegistry,

    FeeTierConfig, // FeeTierConfig struct with thresholds and rates

    UserVolumeHistory(Address), // user_address -> Vec<UserVolumeEntry>
    UserTierCache(Address),     // user_address -> UserTierData

    // Deprecated - kept for backward compatibility during migration
    ProtocolFeeFraction,
    MaxManagerFeeFraction,

    ProtocolFeeRecipient, // Address where protocol fees are sent
    MinimumFeeThreshold,  // Universal minimum fee threshold (immutable)

    // Flat fees (new approach)
    ProtocolFeeAmount,
    MaxManagerFeeAmount,
    MinimumSharesForFeeCollection,

    IndexContractWASM, // wasm of the Index Fund contract
    TokenContractWASM, // wasm of the Index Token contract

    ContractSequence(Address),
    // Index registry storage
    DeployedIndexes(Address), // manager -> Vec<Address>
    AllDeployedIndexes,       // global registry -> Vec<Address>

    // Fee control per index
    IndexFeeEnabled(Address), // index_address -> bool (fee enabled status)

    // paused
    IsKilledCreate,
}

generate_instance_storage_getter_and_setter!(swap_utility, DataKey::SwapUtility, Address);

generate_instance_storage_getter_and_setter!(oracle_registry, DataKey::OracleRegistry, Address);

generate_instance_storage_getter_and_setter!(
    fee_tier_config,
    DataKey::FeeTierConfig,
    FeeTierConfig
);

pub fn get_fee_tier_config_with_default(env: &Env) -> FeeTierConfig {
    let mut tier_rates = Map::new(env);
    tier_rates.set(0u128, 75u32);
    tier_rates.set(10000_0000000u128, 50u32);
    tier_rates.set(100000_0000000u128, 25u32);
    tier_rates.set(1000000_0000000u128, 10u32);

    FeeTierConfig { tier_rates }
}

pub fn get_user_volume_history(env: &Env, user: &Address) -> Vec<UserVolumeEntry> {
    let key = DataKey::UserVolumeHistory(user.clone());
    match env.storage().persistent().get(&key) {
        Some(history) => {
            bump_persistent(env, &key);
            history
        }
        None => Vec::new(env),
    }
}

pub fn add_user_volume_entry(env: &Env, user: &Address, usd_amount: u128, index_address: &Address) {
    let key = DataKey::UserVolumeHistory(user.clone());
    let current_time = env.ledger().timestamp();
    let cutoff_time = current_time.saturating_sub(30 * 24 * 60 * 60);

    let mut history = get_user_volume_history(env, user);

    let new_entry = UserVolumeEntry {
        timestamp: current_time,
        usd_amount,
        index_address: index_address.clone(),
    };
    history.push_back(new_entry);

    let mut cleaned_history = Vec::new(env);
    for entry in history.iter() {
        if entry.timestamp >= cutoff_time {
            cleaned_history.push_back(entry);
        }
    }

    env.storage().persistent().set(&key, &cleaned_history);
    bump_persistent(env, &key);

    let new_total_volume = get_user_30_day_volume(env, user);
    crate::tiers::TierCalculator::invalidate_user_cache(env, user, new_total_volume);
}

pub fn get_user_30_day_volume(env: &Env, user: &Address) -> u128 {
    let history = get_user_volume_history(env, user);
    let current_time = env.ledger().timestamp();
    let cutoff_time = current_time.saturating_sub(30 * 24 * 60 * 60); // 30 days ago

    let mut total_volume = 0u128;
    for entry in history.iter() {
        if entry.timestamp >= cutoff_time {
            total_volume = total_volume.saturating_add(entry.usd_amount);
        }
    }

    total_volume
}

pub fn get_user_tier_cache(env: &Env, user: &Address) -> Option<UserTierData> {
    let key = DataKey::UserTierCache(user.clone());
    match env.storage().persistent().get(&key) {
        Some(data) => {
            bump_persistent(env, &key);
            Some(data)
        }
        None => None,
    }
}

pub fn set_user_tier_cache(env: &Env, user: &Address, tier_data: &UserTierData) {
    let key = DataKey::UserTierCache(user.clone());
    env.storage().temporary().set(&key, tier_data);
    // bump_persistent(env, &key);
}

pub fn invalidate_user_tier_cache(env: &Env, user: &Address) {
    let key = DataKey::UserTierCache(user.clone());
    env.storage().persistent().remove(&key);
}

pub fn cleanup_old_user_volume_entries(env: &Env, user: &Address) {
    let key = DataKey::UserVolumeHistory(user.clone());
    let current_time = env.ledger().timestamp();
    let cutoff_time = current_time.saturating_sub(30 * 24 * 60 * 60); // 30 days ago

    let history = get_user_volume_history(env, user);
    let mut cleaned_history = Vec::new(env);

    for entry in history.iter() {
        if entry.timestamp >= cutoff_time {
            cleaned_history.push_back(entry);
        }
    }

    if cleaned_history.len() != history.len() {
        env.storage().persistent().set(&key, &cleaned_history);
        bump_persistent(env, &key);
    }
}

// Deprecated - using flat amounts instead of fractions
// generate_instance_storage_getter_and_setter!(
//     protocol_fee_fraction,
//     DataKey::ProtocolFeeFraction,
//     u32
// );
// generate_instance_storage_getter_and_setter!(
//     max_manager_fee_fraction,
//     DataKey::MaxManagerFeeFraction,
//     u32
// );
generate_instance_storage_getter_and_setter!(
    protocol_fee_recipient,
    DataKey::ProtocolFeeRecipient,
    Address
);
generate_instance_storage_getter_and_setter!(
    minimum_fee_threshold,
    DataKey::MinimumFeeThreshold,
    u128
);

// Flat fees storage
generate_instance_storage_getter_and_setter_with_default!(
    protocol_fee_amount,
    DataKey::ProtocolFeeAmount,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    max_manager_fee_amount,
    DataKey::MaxManagerFeeAmount,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    minimum_shares_for_fee_collection,
    DataKey::MinimumSharesForFeeCollection,
    u128,
    25_000_000_000 // 25k tokens
);

generate_instance_storage_getter_and_setter!(
    index_contract_wasm,
    DataKey::IndexContractWASM,
    BytesN<32>
);
generate_instance_storage_getter_and_setter!(
    token_contract_wasm,
    DataKey::TokenContractWASM,
    BytesN<32>
);

// paused ops
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_create,
    DataKey::IsKilledCreate,
    bool,
    false
);

pub(crate) fn get_contract_sequence(env: &Env, manager: Address) -> u32 {
    let key = DataKey::ContractSequence(manager);
    match env.storage().persistent().get(&key) {
        Some(sequence) => {
            bump_persistent(env, &key);
            sequence
        }
        None => 0,
    }
}

pub(crate) fn set_contract_sequence(env: &Env, manager: Address, sequence: u32) {
    let key = DataKey::ContractSequence(manager);
    env.storage().persistent().set(&key, &sequence);
    bump_persistent(env, &key);
}

// Index registry functions
pub fn add_deployed_index(env: &Env, manager: &Address, index_address: &Address) {
    // Add to manager's list
    let manager_key: DataKey = DataKey::DeployedIndexes(manager.clone());
    let mut manager_indexes: Vec<Address> = match env.storage().persistent().get(&manager_key) {
        Some(indexes) => indexes,
        None => Vec::new(env),
    };
    manager_indexes.push_back(index_address.clone());
    env.storage()
        .persistent()
        .set(&manager_key, &manager_indexes);
    bump_persistent(env, &manager_key);

    // Add to global list
    let global_key = DataKey::AllDeployedIndexes;
    let mut all_indexes: Vec<Address> = match env.storage().persistent().get(&global_key) {
        Some(indexes) => indexes,
        None => Vec::new(env),
    };
    all_indexes.push_back(index_address.clone());
    env.storage().persistent().set(&global_key, &all_indexes);
    bump_persistent(env, &global_key);
}

pub fn get_deployed_indexes(env: &Env, manager: &Address) -> Vec<Address> {
    let key = DataKey::DeployedIndexes(manager.clone());
    match env.storage().persistent().get(&key) {
        Some(indexes) => {
            bump_persistent(env, &key);
            indexes
        }
        None => Vec::new(env),
    }
}

pub fn get_all_deployed_indexes(env: &Env) -> Vec<Address> {
    let key = DataKey::AllDeployedIndexes;
    match env.storage().persistent().get(&key) {
        Some(indexes) => {
            bump_persistent(env, &key);
            indexes
        }
        None => Vec::new(env),
    }
}

// Index fee control functions
/// Gets fee enabled status for a specific index
/// Returns true if fees are enabled, defaults to true for new indexes
pub fn get_index_fee_enabled(env: &Env, index_address: &Address) -> bool {
    let key = DataKey::IndexFeeEnabled(index_address.clone());
    match env.storage().persistent().get::<DataKey, bool>(&key) {
        Some(enabled) => {
            bump_persistent(env, &key);
            enabled
        }
        None => true, // Default to enabled for new indexes
    }
}

/// Sets fee enabled status for a specific index
pub fn set_index_fee_enabled(env: &Env, index_address: &Address, enabled: bool) {
    let key = DataKey::IndexFeeEnabled(index_address.clone());
    env.storage().persistent().set(&key, &enabled);
    bump_persistent(env, &key);
}
