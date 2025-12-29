use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, Map, String, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

#[derive(Clone)]
#[contracttype]
enum DataKey {
    FeeToken,

    MintFee,
    RedeemFee,

    ProtocolFee(Address), // token address > protocol fees to collect

    // paused ops
    IsKilledFee,
}

pub fn get_fee_token(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::FeeToken) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn put_fee_token(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::FeeToken, &contract)
}

generate_instance_storage_getter_and_setter_with_default!(
    mint_fee,
    DataKey::MintFee,
    u128,
    1_0000000
);
generate_instance_storage_getter_and_setter_with_default!(
    redeem_fee,
    DataKey::RedeemFee,
    u128,
    1_0000000
);

pub fn set_protocol_fee(e: &Env, addr: Address, amount: u128) {
    let key = DataKey::ProtocolFee(addr);
    e.storage().persistent().set(&key, &amount);
    bump_persistent(e, &key);
}

pub fn get_protocol_fee(e: &Env, addr: Address) -> u128 {
    let key = DataKey::ProtocolFee(addr);
    match e.storage().persistent().get::<DataKey, u128>(&key) {
        Some(balance) => {
            bump_persistent(e, &key);
            balance
        }
        None => 0,
    }
}

generate_instance_storage_getter_and_setter_with_default!(
    is_killed_fee,
    DataKey::IsKilledFee,
    bool,
    false
);
