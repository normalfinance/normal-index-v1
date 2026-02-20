use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, Symbol, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_setter,
};

use crate::errors::IndexFundFactoryError;

/********** Storage Types **********/

/// Snapshot of factory configuration returned by query methods.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexFundFactoryConfig {
    /// WASM hash used when deploying index-fund contracts.
    pub index_contract_wasm: BytesN<32>,
    /// WASM hash used when deploying index-token contracts.
    pub index_token_wasm: BytesN<32>,
    /// Adapter-registry contract used by newly deployed funds.
    pub adapter_registry: Address,
}

/********** Storage Key Types **********/

/// Instance key for the index-fund contract WASM hash.
const KEY_INDEX_CONTRACT_WASM: &str = "IndexContractWASM";
/// Instance key for the index-token contract WASM hash.
const KEY_INDEX_TOKEN_WASM: &str = "IndexTokenWASM";
/// Instance key for the adapter-registry contract address.
const KEY_ADAPTER_REGISTRY: &str = "AdapterRegistry";

/// Persistent storage keys for factory-managed data.
///
#[derive(Clone)]
#[contracttype]
enum DataKey {
    /// Global incrementing deployment id.
    ContractSequence,
    /// Deployment id -> index-fund address.
    DeployedIndex(u32),
    /// Manager address -> list of deployed index-fund addresses.
    DeployedIndexesByManager(Address),
}

/********** Storage **********/

generate_instance_storage_getter_and_setter!(
    index_contract_wasm,
    KEY_INDEX_CONTRACT_WASM,
    BytesN<32>
);
generate_instance_storage_getter_and_setter!(index_token_wasm, KEY_INDEX_TOKEN_WASM, BytesN<32>);
generate_instance_storage_getter_and_setter!(adapter_registry, KEY_ADAPTER_REGISTRY, Address);

/// Returns the current deployment sequence counter.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
///
/// # Returns
/// - `u32`: Current deployment sequence.
pub(crate) fn get_contract_sequence(env: &Env) -> u32 {
    let key = DataKey::ContractSequence;
    match env.storage().persistent().get(&key) {
        Some(sequence) => {
            bump_persistent(env, &key);
            sequence
        }
        None => 0,
    }
}

/// Sets the deployment sequence counter.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `sequence` (`u32`): New sequence value.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub(crate) fn set_contract_sequence(env: &Env, sequence: u32) {
    let key = DataKey::ContractSequence;
    env.storage().persistent().set(&key, &sequence);
    bump_persistent(env, &key);
}

/// Registers a newly deployed index both globally and under its manager.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `sequence` (`&u32`): Deployment id.
/// - `manager` (`&Address`): Manager address for ownership index.
/// - `index_address` (`&Address`): Deployed index contract address.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn add_deployed_index(env: &Env, sequence: &u32, manager: &Address, index_address: &Address) {
    // Add to global map
    let global_map_key: DataKey = DataKey::DeployedIndex(sequence.clone());
    match env
        .storage()
        .persistent()
        .get::<DataKey, Address>(&global_map_key)
    {
        Some(_) => panic_with_error!(env, IndexFundFactoryError::IndexAlreadyExists),
        None => {
            env.storage()
                .persistent()
                .set(&global_map_key, &index_address);
            bump_persistent(env, &global_map_key);
        }
    }

    // Add to manager's list
    let manager_key: DataKey = DataKey::DeployedIndexesByManager(manager.clone());
    let mut manager_indexes: Vec<Address> = match env.storage().persistent().get(&manager_key) {
        Some(index_ids) => index_ids,
        None => Vec::new(env),
    };
    manager_indexes.push_back(index_address.clone());
    env.storage()
        .persistent()
        .set(&manager_key, &manager_indexes);
    bump_persistent(env, &manager_key);
}

/// Returns the deployed index address for a deployment sequence id.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `sequence` (`&u32`): Deployment id.
///
/// # Returns
/// - `Address`: Deployed index contract address.
pub fn get_deployed_index(env: &Env, sequence: &u32) -> Address {
    let key = DataKey::DeployedIndex(sequence.clone());
    match env.storage().persistent().get(&key) {
        Some(index_address) => {
            bump_persistent(env, &key);
            index_address
        }
        None => panic_with_error!(env, IndexFundFactoryError::IndexNotFound),
    }
}

/// Returns all deployed index addresses for a manager.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `manager` (`&Address`): Manager address.
///
/// # Returns
/// - `Vec<Address>`: Deployed indexes associated with `manager`.
pub fn get_deployed_indexes_by_manager(env: &Env, manager: &Address) -> Vec<Address> {
    let key = DataKey::DeployedIndexesByManager(manager.clone());
    match env.storage().persistent().get(&key) {
        Some(indexes) => {
            bump_persistent(env, &key);
            indexes
        }
        None => Vec::new(env),
    }
}
