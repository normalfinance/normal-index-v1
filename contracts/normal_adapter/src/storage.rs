use paste::paste;
use soroban_sdk::{contracttype, log, panic_with_error, Address, Env, Map, Vec};
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
    Treasury,
}

/********** Storage **********/

generate_instance_storage_getter_and_setter!(admin, DataKey::Admin, Address);
generate_instance_storage_getter_and_setter!(treasury, DataKey::Treasury, Address);
