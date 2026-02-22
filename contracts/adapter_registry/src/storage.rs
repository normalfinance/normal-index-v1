use soroban_sdk::{contracttype, Address, Env, Map, Symbol, Vec};

/********** Storage Key Types **********/

/// Instance storage keys for adapter-registry mappings.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Adapter address keyed by symbolic adapter name.
    AdapterByName(Symbol),
    /// Adapter name keyed by adapter contract address.
    NameByAdapter(Address),
    /// List of all known adapter names.
    AdapterNames,
}

/********** Storage **********/

/// Returns adapter address for a given adapter name.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `name` (`&Symbol`): Adapter name.
///
/// # Returns
/// - `Option<Address>`: Adapter contract address if present.
pub fn get_adapter_by_name(e: &Env, name: &Symbol) -> Option<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::AdapterByName(name.clone()))
}

/// Stores adapter address for a given adapter name.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `name` (`&Symbol`): Adapter name.
/// - `adapter` (`&Address`): Adapter contract address.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn set_adapter_by_name(e: &Env, name: &Symbol, adapter: &Address) {
    e.storage()
        .persistent()
        .set(&DataKey::AdapterByName(name.clone()), adapter);
}

/// Returns adapter name for a given adapter address.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `adapter` (`&Address`): Adapter contract address.
///
/// # Returns
/// - `Option<Symbol>`: Adapter name if present.
pub fn get_name_by_adapter(e: &Env, adapter: &Address) -> Option<Symbol> {
    e.storage()
        .persistent()
        .get(&DataKey::NameByAdapter(adapter.clone()))
}

/// Stores adapter name for a given adapter address.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `adapter` (`&Address`): Adapter contract address.
/// - `name` (`&Symbol`): Adapter name.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn set_name_by_adapter(e: &Env, adapter: &Address, name: &Symbol) {
    e.storage()
        .persistent()
        .set(&DataKey::NameByAdapter(adapter.clone()), name);
}

/// Removes reverse mapping from adapter address to name.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `adapter` (`&Address`): Adapter contract address to remove.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn remove_name_by_adapter(e: &Env, adapter: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::NameByAdapter(adapter.clone()));
}

/// Returns all registered adapter names.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
///
/// # Returns
/// - `Vec<Symbol>`: Registered adapter names.
pub fn get_adapter_names(e: &Env) -> Vec<Symbol> {
    e.storage()
        .persistent()
        .get(&DataKey::AdapterNames)
        .unwrap_or(Vec::new(e))
}

/// Adds an adapter name to the name list if it is not already present.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `name` (`&Symbol`): Adapter name to add.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn add_adapter_name(e: &Env, name: &Symbol) {
    let mut names = get_adapter_names(e);
    let len = names.len();
    for i in 0..len {
        if names.get_unchecked(i) == *name {
            return;
        }
    }
    names.push_back(name.clone());
    e.storage().persistent().set(&DataKey::AdapterNames, &names);
}

/// Returns a complete name-to-address map for all registered adapters.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
///
/// # Returns
/// - `Map<Symbol, Address>`: Mapping of adapter names to contract addresses.
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
