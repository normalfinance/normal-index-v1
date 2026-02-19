use soroban_sdk::{contracttype, Address, Env, Map, Symbol, Vec};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    AdapterByName(Symbol),
    NameByAdapter(Address),
    AdapterNames,
}

pub fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::Admin)
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_adapter_by_name(e: &Env, name: &Symbol) -> Option<Address> {
    e.storage()
        .instance()
        .get(&DataKey::AdapterByName(name.clone()))
}

pub fn set_adapter_by_name(e: &Env, name: &Symbol, adapter: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::AdapterByName(name.clone()), adapter);
}

pub fn get_name_by_adapter(e: &Env, adapter: &Address) -> Option<Symbol> {
    e.storage()
        .instance()
        .get(&DataKey::NameByAdapter(adapter.clone()))
}

pub fn set_name_by_adapter(e: &Env, adapter: &Address, name: &Symbol) {
    e.storage()
        .instance()
        .set(&DataKey::NameByAdapter(adapter.clone()), name);
}

pub fn remove_name_by_adapter(e: &Env, adapter: &Address) {
    e.storage()
        .instance()
        .remove(&DataKey::NameByAdapter(adapter.clone()));
}

pub fn get_adapter_names(e: &Env) -> Vec<Symbol> {
    e.storage()
        .instance()
        .get(&DataKey::AdapterNames)
        .unwrap_or(Vec::new(e))
}

pub fn add_adapter_name(e: &Env, name: &Symbol) {
    let mut names = get_adapter_names(e);
    let len = names.len();
    for i in 0..len {
        if names.get_unchecked(i) == *name {
            return;
        }
    }
    names.push_back(name.clone());
    e.storage().instance().set(&DataKey::AdapterNames, &names);
}

pub fn get_all_adapters(e: &Env) -> Map<Symbol, Address> {
    let mut adapters = Map::new(e);
    let names = get_adapter_names(e);
    let len = names.len();
    for i in 0..len {
        let name = names.get_unchecked(i);
        if let Some(address) = get_adapter_by_name(e, &name) {
            adapters.set(name, address);
        }
    }
    adapters
}
