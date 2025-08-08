use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, String, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default, generate_instance_storage_setter,
};

// FROM SOROSWAP
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexDistribution {
    pub protocol_id: String,
    pub path: Vec<Address>,
    pub parts: u32,
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Aggregator, // DEX Aggregator
    Router,     // DEX Router
    ProtocolFeeFraction,
    MaxManagerFeeFraction,
    ProtocolFeeRecipient, // Address where protocol fees are sent
    IndexContractWASM,
    ContractSequence(Address),
    // Index registry storage
    DeployedIndexes(Address), // operator -> Vec<Address>
    AllDeployedIndexes,       // global registry -> Vec<Address>

    // paused
    IsKilledCreate,
}

generate_instance_storage_getter_and_setter!(aggregator, DataKey::Aggregator, Address);
generate_instance_storage_getter_and_setter!(router, DataKey::Router, Address);
generate_instance_storage_getter_and_setter!(
    protocol_fee_fraction,
    DataKey::ProtocolFeeFraction,
    u32
);
generate_instance_storage_getter_and_setter!(
    max_manager_fee_fraction,
    DataKey::MaxManagerFeeFraction,
    u32
);
generate_instance_storage_getter_and_setter!(
    protocol_fee_recipient,
    DataKey::ProtocolFeeRecipient,
    Address
);
generate_instance_storage_getter_and_setter!(
    fee_contract_wasm,
    DataKey::IndexContractWASM,
    BytesN<32>
);

// paused ops
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_create,
    DataKey::IsKilledCreate,
    bool,
    false
);

pub(crate) fn get_contract_sequence(env: &Env, operator: Address) -> u32 {
    let key = DataKey::ContractSequence(operator);
    match env.storage().persistent().get(&key) {
        Some(sequence) => {
            bump_persistent(env, &key);
            sequence
        }
        None => 0,
    }
}

pub(crate) fn set_contract_sequence(env: &Env, operator: Address, sequence: u32) {
    let key = DataKey::ContractSequence(operator);
    env.storage().persistent().set(&key, &sequence);
    bump_persistent(env, &key);
}

// Index registry functions
pub fn add_deployed_index(env: &Env, operator: &Address, index_address: &Address) {
    // Add to operator's list
    let operator_key = DataKey::DeployedIndexes(operator.clone());
    let mut operator_indexes: Vec<Address> = match env.storage().persistent().get(&operator_key) {
        Some(indexes) => indexes,
        None => Vec::new(env),
    };
    operator_indexes.push_back(index_address.clone());
    env.storage()
        .persistent()
        .set(&operator_key, &operator_indexes);
    bump_persistent(env, &operator_key);

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

pub fn get_deployed_indexes(env: &Env, operator: &Address) -> Vec<Address> {
    let key = DataKey::DeployedIndexes(operator.clone());
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
