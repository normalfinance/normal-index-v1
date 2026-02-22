use paste::paste;
use soroban_sdk::{panic_with_error, Address, Env, String, Symbol};
use utils::bump::bump_instance;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

/********** Storage Key Types **********/

/// Instance key for initialization.
const KEY_INITIALIZED: &str = "Initialized";
/// Instance key for adapter admin.
const KEY_ADMIN: &str = "Admin";
/// Instance key for protocol identifier string.
const KEY_PROTOCOL_ID: &str = "ProtocolId";
/// Instance key for protocol contract address.
const KEY_PROTOCOL_ADDRESS: &str = "ProtocolAddress";

/********** Storage **********/

generate_instance_storage_getter_and_setter_with_default!(
    initialized,
    KEY_INITIALIZED,
    bool,
    false
);
generate_instance_storage_getter_and_setter!(admin, KEY_ADMIN, Address);
generate_instance_storage_getter_and_setter!(protocol_id, KEY_PROTOCOL_ID, String);
generate_instance_storage_getter_and_setter!(protocol_address, KEY_PROTOCOL_ADDRESS, Address);
