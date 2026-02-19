use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, Address, Env, Map, Symbol,
};

use crate::errors::AdapterRegistryError;
use crate::interface::AdapterRegistryTrait;
use crate::storage;

contractmeta!(
    key = "Description",
    val = "A registry tracking supported adapters for use with Index Funds"
);

#[contract]
pub struct AdapterRegistry;

#[contractimpl]
impl AdapterRegistry {
    pub fn __constructor(e: Env, admin: Address) {
        if storage::get_admin(&e).is_some() {
            panic_with_error!(&e, AdapterRegistryError::Unauthorized);
        }
        storage::set_admin(&e, &admin);
    }
}

#[contractimpl]
impl AdapterRegistryTrait for AdapterRegistry {
    fn set_adapter(e: Env, admin: Address, name: Symbol, adapter: Address) {
        admin.require_auth();

        match storage::get_admin(&e) {
            Some(owner) if owner == admin => {}
            _ => panic_with_error!(&e, AdapterRegistryError::Unauthorized),
        }

        if let Some(assigned_name) = storage::get_name_by_adapter(&e, &adapter) {
            if assigned_name != name {
                panic_with_error!(&e, AdapterRegistryError::AdapterAddressAlreadyAssigned);
            }
        }

        if let Some(old_adapter) = storage::get_adapter_by_name(&e, &name) {
            if old_adapter != adapter {
                storage::remove_name_by_adapter(&e, &old_adapter);
            }
        }

        storage::set_adapter_by_name(&e, &name, &adapter);
        storage::set_name_by_adapter(&e, &adapter, &name);
        storage::add_adapter_name(&e, &name);
    }

    fn get_adapter(e: Env, name: Symbol) -> Address {
        match storage::get_adapter_by_name(&e, &name) {
            Some(address) => address,
            None => panic_with_error!(&e, AdapterRegistryError::AdapterNameNotFound),
        }
    }

    fn get_adapter_safe(e: Env, name: Symbol) -> Option<Address> {
        storage::get_adapter_by_name(&e, &name)
    }

    fn get_adapter_name(e: Env, adapter: Address) -> Symbol {
        match storage::get_name_by_adapter(&e, &adapter) {
            Some(name) => name,
            None => panic_with_error!(&e, AdapterRegistryError::AdapterAddressNotFound),
        }
    }

    fn get_adapters(e: Env) -> Map<Symbol, Address> {
        storage::get_all_adapters(&e)
    }
}
