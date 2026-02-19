use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, String};
use utils::bump::bump_instance;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_setter,
};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    ProtocolId,
    ProtocolAddress,
    ProtocolQuoteToken, // USDC
}

/********** Storage **********/

generate_instance_storage_getter_and_setter!(admin, DataKey::Admin, Address);
generate_instance_storage_getter_and_setter!(protocol_id, DataKey::ProtocolId, String);
generate_instance_storage_getter_and_setter!(protocol_address, DataKey::ProtocolAddress, Address);
generate_instance_storage_getter_and_setter!(
    protocol_quote_token,
    DataKey::ProtocolQuoteToken,
    Address
);
