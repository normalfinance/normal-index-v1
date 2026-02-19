use paste::paste;
use soroban_sdk::{panic_with_error, Address, Env, String, Symbol};
use utils::bump::bump_instance;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_setter,
};

/********** Storage Key Types **********/

const KEY_ADMIN: &str = "Admin";
const KEY_PROTOCOL_ID: &str = "ProtocolId";
const KEY_PROTOCOL_ADDRESS: &str = "ProtocolAddress";

/********** Storage **********/

generate_instance_storage_getter_and_setter!(admin, KEY_ADMIN, Address);
generate_instance_storage_getter_and_setter!(protocol_id, KEY_PROTOCOL_ID, String);
generate_instance_storage_getter_and_setter!(protocol_address, KEY_PROTOCOL_ADDRESS, Address);
