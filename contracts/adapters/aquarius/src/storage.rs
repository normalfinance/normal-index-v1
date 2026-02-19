use paste::paste;
use soroban_sdk::{contracttype, log, panic_with_error, Address, Env, String};
use utils::bump::{bump_instance, bump_persistent};
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    ProtocolId,
    ProtocolAddress,
}

/********** Storage **********/

generate_instance_storage_getter_and_setter!(admin, DataKey::Admin, Address);
generate_instance_storage_getter_and_setter!(protocol_id, DataKey::ProtocolId, String);
generate_instance_storage_getter_and_setter!(protocol_address, DataKey::ProtocolAddress, Address);
