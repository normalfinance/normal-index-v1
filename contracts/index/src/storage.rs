use paste::paste;
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Map, Symbol, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::constant::THIRTY_DAY;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Factory,
    TokenIndex, //

    TotalShares,

    BaseNAV, // The Net Asset Value (NAV) at the inception of the index - what the creator deposits (e.g. $1,000)
    InitialPrice, // The price assigned to the index at inception (e.g. $100)

    Component(Address), // Map of token address to Component
    ComponentBalance(Address),

    Public, // Private indexes are mutable and can only be minted by the admin and whitelist. Pubilic indexes are immutabel and can be minted by anyone

    ManagerFeeFraction, // A custom annual fee set by the admin

    // Revenue Share
    ManagerAddress,          // Address of the index manager who receives fees
    ProtocolFeeRecipient,    // Address where protocol fees are sent
    AccumulatedManagerFees,  // Total manager fees accumulated but not yet distributed
    AccumulatedProtocolFees, // Total protocol fees accumulated but not yet distributed
    LastFeeCollection,       // Timestamp of last fee collection

    Whitelist(Address), // List of accounts explicitly allowed to mint the index
    Blacklist(Address), // List of accounts blocked from minting the index

    RebalanceThreshold, // Minimum amount of time that must pass before the index can be rebalanced again

    LastRebalanceTs, // The ts when the index was last rebalanced
    LastUpdatedTs,   // The ts when the index was last updated (any property)

    // Metrics
    TotalMints,
    TotalRedemptions,
    TotalFees,

    // Paused operations
    IsKilledMint,
    IsKilledRedeem,
    IsKilledRebalance,

    // Component registry
    ComponentRegistry, // Vec<Address> - list of all component addresses

    // Rebalancing authorities (for private indexes)
    RebalanceAuthority(Address), // Address -> bool mapping for rebalance authorities
    RebalanceAuthorityRegistry, // Vec<Address> - list of all rebalance authority addresses
}

generate_instance_storage_getter_and_setter_with_default!(
    factory,
    DataKey::Factory,
    Address,
    Address::from_str(&Env::default(), "")
);

// Financial Configuration
generate_instance_storage_getter_and_setter_with_default!(base_nav, DataKey::BaseNAV, i128, 0);
generate_instance_storage_getter_and_setter_with_default!(
    initial_price,
    DataKey::InitialPrice,
    i128,
    0
);

// State
generate_instance_storage_getter_and_setter_with_default!(
    total_shares,
    DataKey::TotalShares,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    manager_fee_fraction,
    DataKey::ManagerFeeFraction,
    u32,
    0
);

// Revenue Share storage
generate_instance_storage_getter_and_setter_with_default!(
    manager_address,
    DataKey::ManagerAddress,
    Address,
    Address::from_str(&Env::default(), "")
);
generate_instance_storage_getter_and_setter_with_default!(
    protocol_fee_recipient,
    DataKey::ProtocolFeeRecipient,
    Address,
    Address::from_str(&Env::default(), "")
);
generate_instance_storage_getter_and_setter_with_default!(
    accumulated_manager_fees,
    DataKey::AccumulatedManagerFees,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    accumulated_protocol_fees,
    DataKey::AccumulatedProtocolFees,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    last_fee_collection,
    DataKey::LastFeeCollection,
    u64,
    0
);

generate_instance_storage_getter_and_setter_with_default!(public, DataKey::Public, bool, false);
generate_instance_storage_getter_and_setter_with_default!(
    rebalance_threshold,
    DataKey::RebalanceThreshold,
    u64,
    THIRTY_DAY
);

// Whitelist/Blacklist functions
// Note: These use manual implementation (not macros) because they are keyed storage patterns
// that require persistent storage, custom TTL management, and Address-based keys.
// This follows the same pattern as Component(Address) and ComponentBalance(Address) storage.

/// Checks if an address is whitelisted
/// Returns true if whitelisted, false if not (missing entries are treated as not whitelisted)
pub fn get_whitelist_status(e: &Env, address: &Address) -> bool {
    let key = DataKey::Whitelist(address.clone());
    match e.storage().persistent().get::<DataKey, Address>(&key) {
        Some(_) => {
            bump_persistent(e, &key);
            true
        }
        None => false,
    }
}

/// Sets whitelist status for an address
/// If status is true, adds the address to whitelist; if false, removes it
pub fn set_whitelist_status(e: &Env, address: &Address, status: bool) {
    let key = DataKey::Whitelist(address.clone());
    if status {
        e.storage().persistent().set(&key, address);
        e.storage().persistent().extend_ttl(&key, 100000, 100000);
    } else {
        e.storage().persistent().remove(&key);
    }
}

/// Checks if an address is blacklisted
/// Returns true if blacklisted, false if not (missing entries are treated as not blacklisted)
pub fn get_blacklist_status(e: &Env, address: &Address) -> bool {
    let key = DataKey::Blacklist(address.clone());
    match e.storage().persistent().get::<DataKey, Address>(&key) {
        Some(_) => {
            bump_persistent(e, &key);
            true
        }
        None => false,
    }
}

/// Sets blacklist status for an address  
/// If status is true, adds the address to blacklist; if false, removes it
pub fn set_blacklist_status(e: &Env, address: &Address, status: bool) {
    let key = DataKey::Blacklist(address.clone());
    if status {
        e.storage().persistent().set(&key, address);
        e.storage().persistent().extend_ttl(&key, 100000, 100000);
    } else {
        e.storage().persistent().remove(&key);
    }
}


/// Checks if an address has rebalance authority for private indexes
/// Returns true if authorized, false if not (missing entries are treated as not authorized)
pub fn get_rebalance_authority_status(e: &Env, address: &Address) -> bool {
    let key = DataKey::RebalanceAuthority(address.clone());
    match e.storage().persistent().get::<DataKey, Address>(&key) {
        Some(_) => {
            bump_persistent(e, &key);
            true
        }
        None => false,
    }
}

pub fn set_rebalance_authority_status(e: &Env, address: &Address, status: bool) {
    let key = DataKey::RebalanceAuthority(address.clone());
    if status {
        e.storage().persistent().set(&key, address);
        e.storage().persistent().extend_ttl(&key, 100000, 100000);
        add_rebalance_authority_to_registry(e, address.clone());
    } else {
        e.storage().persistent().remove(&key);
        remove_rebalance_authority_from_registry(e, address.clone());
    }
}

// Rebalance Authority Registry Management Functions

/// Gets the list of all rebalance authorities
pub fn get_rebalance_authority_registry(e: &Env) -> Vec<Address> {
    let key = DataKey::RebalanceAuthorityRegistry;
    match e.storage().persistent().get(&key) {
        Some(registry) => {
            bump_persistent(e, &key);
            registry
        }
        None => Vec::new(e),
    }
}

pub fn add_rebalance_authority_to_registry(e: &Env, address: Address) {
    let key = DataKey::RebalanceAuthorityRegistry;
    let mut registry: Vec<Address> = match e.storage().persistent().get(&key) {
        Some(reg) => reg,
        None => Vec::new(e),
    };

    for existing_address in registry.iter() {
        if existing_address == address {
            return; 
        }
    }

    registry.push_back(address);
    e.storage().persistent().set(&key, &registry);
    bump_persistent(e, &key);
}

pub fn remove_rebalance_authority_from_registry(e: &Env, address: Address) {
    let key = DataKey::RebalanceAuthorityRegistry;
    let mut registry: Vec<Address> = match e.storage().persistent().get(&key) {
        Some(reg) => reg,
        None => return, 
    };

    let mut new_registry = Vec::new(e);
    for existing_address in registry.iter() {
        if existing_address != address {
            new_registry.push_back(existing_address);
        }
    }

    e.storage().persistent().set(&key, &new_registry);
    bump_persistent(e, &key);
}


pub fn get_all_rebalance_authorities(e: &Env) -> Vec<Address> {
    let registry = get_rebalance_authority_registry(e);
    let mut active_authorities = Vec::new(e);
    
    for address in registry.iter() {
        if get_rebalance_authority_status(e, &address) {
            active_authorities.push_back(address);
        }
    }
    
    active_authorities
}

// Timestamps
generate_instance_storage_getter_and_setter_with_default!(
    last_rebalance_ts,
    DataKey::LastRebalanceTs,
    u64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    last_updated_ts,
    DataKey::LastUpdatedTs,
    u64,
    0
);

// Component Balance

pub fn get_component_balance(e: &Env, token: Address) -> u128 {
    let key = DataKey::ComponentBalance(token);
    match e.storage().persistent().get::<DataKey, u128>(&key) {
        Some(balance) => {
            bump_persistent(e, &key);
            balance
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

// Component

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    pub asset: Symbol,
    pub weight: u128,
}

pub fn get_all_components(e: &Env) -> Map<Address, Component> {
    let mut components_map = Map::new(e);

    // Get the list of component addresses from registry
    let component_addresses = get_component_registry(e);

    // Iterate through each component address and get its data
    for address in component_addresses.iter() {
        match get_component_safe(e, address.clone()) {
            Some(component) => {
                components_map.set(address, component);
            }
            None => {
                // Skip components that no longer exist
                continue;
            }
        }
    }

    components_map
}

// Helper function to get component registry
pub fn get_component_registry(e: &Env) -> Vec<Address> {
    let key = DataKey::ComponentRegistry;
    match e.storage().persistent().get(&key) {
        Some(registry) => {
            bump_persistent(e, &key);
            registry
        }
        None => Vec::new(e),
    }
}

// Helper function to get component without panicking
pub fn get_component_safe(e: &Env, token: Address) -> Option<Component> {
    let key = DataKey::Component(token);
    match e.storage().persistent().get::<DataKey, Component>(&key) {
        Some(component) => {
            bump_persistent(e, &key);
            Some(component)
        }
        None => None,
    }
}

pub fn get_component(e: &Env, token: Address) -> Component {
    let key = DataKey::Component(token);
    match e.storage().persistent().get::<DataKey, Component>(&key) {
        Some(component) => {
            bump_persistent(e, &key);
            component
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn set_component(env: &Env, token: Address, component: Component) {
    let key = DataKey::Component(token.clone());
    env.storage().persistent().set(&key, &component);
    env.storage().persistent().extend_ttl(&key, 100000, 100000);
}

pub fn remove_component(env: &Env, token: Address) {
    let key = DataKey::Component(token.clone());
    env.storage().persistent().remove(&key);
    
    remove_component_from_registry(env, token.clone());
    
    let balance_key = DataKey::ComponentBalance(token);
    env.storage().persistent().remove(&balance_key);
}

pub fn update_component_weight(env: &Env, token: Address, new_weight: u128) {
    let mut component = get_component(env, token.clone());
    component.weight = new_weight;
    set_component(env, token, component);
}

// Proper implementation of get_all_component_balances
pub fn get_all_component_balances(e: &Env) -> Map<Address, u128> {
    let mut balances_map = Map::new(e);

    // Get the list of component addresses from registry
    let component_addresses = get_component_registry(e);

    // Iterate through each component address and get its balance
    for address in component_addresses.iter() {
        match get_component_balance_safe(e, address.clone()) {
            Some(balance) => {
                balances_map.set(address, balance);
            }
            None => {
                // If no balance stored, default to 0
                balances_map.set(address, 0u128);
            }
        }
    }

    balances_map
}

// Helper function to get component balance without panicking
pub fn get_component_balance_safe(e: &Env, token: Address) -> Option<u128> {
    let key = DataKey::ComponentBalance(token);
    match e.storage().persistent().get::<DataKey, u128>(&key) {
        Some(balance) => {
            bump_persistent(e, &key);
            Some(balance)
        }
        None => None,
    }
}

pub fn set_component_balance(env: &Env, token: Address, balance: u128) {
    let key = DataKey::ComponentBalance(token);
    env.storage().persistent().set(&key, &balance);
    env.storage().persistent().extend_ttl(&key, 100000, 100000);
}

// Component registry management functions
pub fn add_component_to_registry(env: &Env, token: Address) {
    let key = DataKey::ComponentRegistry;
    let mut registry: Vec<Address> = match env.storage().persistent().get(&key) {
        Some(reg) => reg,
        None => Vec::new(env),
    };

    // Check if component is already in registry
    for existing_token in registry.iter() {
        if existing_token == token {
            return; // Already exists, don't add duplicate
        }
    }

    // Add new component to registry
    registry.push_back(token);
    env.storage().persistent().set(&key, &registry);
    bump_persistent(env, &key);
}

pub fn remove_component_from_registry(env: &Env, token: Address) {
    let key = DataKey::ComponentRegistry;
    let mut registry: Vec<Address> = match env.storage().persistent().get(&key) {
        Some(reg) => reg,
        None => return, // No registry exists
    };

    // Find and remove the component
    let mut new_registry = Vec::new(env);
    for existing_token in registry.iter() {
        if existing_token != token {
            new_registry.push_back(existing_token);
        }
    }

    env.storage().persistent().set(&key, &new_registry);
    bump_persistent(env, &key);
}

// Helper function to get factory address safely
pub fn get_factory_safe(e: &Env) -> Option<Address> {
    let key = DataKey::Factory;
    match e.storage().instance().get(&key) {
        Some(factory) => {
            bump_instance(e);
            // Check if it's a valid address (not empty string)
            let empty_address = Address::from_str(e, "");
            if factory == empty_address {
                None
            } else {
                Some(factory)
            }
        }
        None => None,
    }
}

// Metrics
generate_instance_storage_getter_and_setter_with_default!(total_fees, DataKey::TotalFees, u128, 0);
generate_instance_storage_getter_and_setter_with_default!(
    total_mints,
    DataKey::TotalMints,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    total_redemptions,
    DataKey::TotalRedemptions,
    u128,
    0
);

// Paused operations
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_mint,
    DataKey::IsKilledMint,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_redeem,
    DataKey::IsKilledRedeem,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_rebalance,
    DataKey::IsKilledRebalance,
    bool,
    false
);

pub fn get_index_vault_amount(e: &Env, token: &Address) -> u128 {
    SorobanTokenClient::new(e, token).balance(&e.current_contract_address()) as u128
}

pub fn get_insurance_vault_amount(e: &Env) -> u128 {
    // Placeholder implementation - return 0 for now
    0
}

pub fn get_token(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::TokenIndex) {
        Some(token) => {
            bump_instance(e);
            token
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn put_token(e: &Env, token: &Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::TokenIndex, token);
}

pub fn get_max_shares(e: &Env) -> u128 {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::TotalShares)
        .unwrap_or(0)
}

pub fn set_max_shares(e: &Env, max_shares: &u128) {
    bump_instance(e);
    e.storage()
        .instance()
        .set(&DataKey::TotalShares, max_shares);
}

pub fn get_unstaking_period(e: &Env) -> u64 {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::LastRebalanceTs)
        .unwrap_or(0)
}

pub fn set_unstaking_period(e: &Env, period: &u64) {
    bump_instance(e);
    e.storage()
        .instance()
        .set(&DataKey::LastRebalanceTs, period);
}

pub fn get_shares_base(e: &Env) -> u128 {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::TotalShares)
        .unwrap_or(0)
}
