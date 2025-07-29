use paste::paste;
use soroban_sdk::{ contracttype, panic_with_error, Address, BytesN, Env, String, Vec };
use utils::bump::{ bump_instance, bump_persistent };
use utils::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter,
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_setter,
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
    Router, // DEX Router
    ProtocolFeeFraction,
    MaxManagerFeeFraction,
    ProtocolFeeRecipient, // Address where protocol fees are sent
    IndexContractWASM,
    ContractSequence(Address),
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
