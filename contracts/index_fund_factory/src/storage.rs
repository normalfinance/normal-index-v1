use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, Map, String, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

// FROM SOROSWAP
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexDistribution {
    pub protocol_id: String,
    pub path: Vec<Address>,
    pub parts: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserVolumeEntry {
    pub timestamp: u64,
    pub usd_amount: u128,
    pub index_address: Address,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    SwapUtility, // DEX Aggregator
    OracleRegistry,

    IndexContractWASM, // wasm of the Index Fund contract

    ContractSequence(Address),
    // Index registry storage
    DeployedIndexes(Address), // manager -> Vec<Address>
    AllDeployedIndexes,       // global registry -> Vec<Address>
}

generate_instance_storage_getter_and_setter!(swap_utility, DataKey::SwapUtility, Address);

generate_instance_storage_getter_and_setter!(oracle_registry, DataKey::OracleRegistry, Address);

generate_instance_storage_getter_and_setter!(
    index_contract_wasm,
    DataKey::IndexContractWASM,
    BytesN<32>
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
