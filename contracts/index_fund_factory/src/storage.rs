use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

use crate::errors::IndexFundFactoryError;

/********** Storage Types **********/

// Factory configuration struct for query methods
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexFundFactoryConfig {
    pub index_contract_wasm: BytesN<32>,
    pub index_token_wasm: BytesN<32>,
    pub adapter_registry: Address,
}

/********** Storage Key Types **********/

const KEY_INDEX_CONTRACT_WASM: &str = "IndexContractWASM";
const KEY_INDEX_TOKEN_WASM: &str = "IndexTokenWASM";
const KEY_ADAPTER_REGISTRY: &str = "AdapterRegistry";

/// Persistent storage keys for factory-managed data.
///
#[derive(Clone)]
#[contracttype]
enum DataKey {
    ///
    ContractSequence,

    DeployedIndex(u32),

    /// manager -> Vec<u32>
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

pub(crate) fn set_contract_sequence(env: &Env, sequence: u32) {
    let key = DataKey::ContractSequence;
    env.storage().persistent().set(&key, &sequence);
    bump_persistent(env, &key);
}

// Index registry functions
pub fn add_deployed_index(env: &Env, sequence: &u32, manager: &Address, index_address: &Address) {
    // Add to global map
    let global_map_key: DataKey = DataKey::DeployedIndex(sequence.clone());
    match env.storage().persistent().get(&global_map_key) {
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
    let mut manager_index_ids: Vec<u32> = match env.storage().persistent().get(&manager_key) {
        Some(index_ids) => index_ids,
        None => Vec::new(env),
    };
    manager_index_ids.push_back(sequence.clone());
    env.storage()
        .persistent()
        .set(&manager_key, &manager_index_ids);
    bump_persistent(env, &manager_key);
}

pub fn get_deployed_index(env: &Env, sequence: &u32) -> Address {
    let key = DataKey::DeployedIndex(sequence.clone());
    match env.storage().persistent().get(&key) {
        Some(index_address) => {
            bump_persistent(env, &key);
            index_address
        }
        None => panic_with_error!(&e, IndexFundFactoryError::IndexNotFound),
    }
}

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
