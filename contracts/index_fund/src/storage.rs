use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Map, Symbol, Vec};

use types::component::Component;
use types::volume::VolumeFeeTier;
use utils::bump::{bump_instance, bump_persistent};
use utils::constant::THIRTY_DAY;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

/********** Storage Key Types **********/

const KEY_FACTORY: &str = "Factory";
const KEY_ADAPTER_REGISTRY: &str = "AdapterRegistry";

/// The token accepted during minting and used to swap, i.e. USDC
const KEY_TOKEN_QUOTE: &str = "TokenQuote";

/// The price assigned to the index at inception (e.g. $100)
const KEY_INITIAL_PRICE: &str = "InitialPrice";

/// Private indexes are mutable and can only be minted by the admin and whitelist. Pubilic indexes are immutabel and can be minted by anyone
const KEY_PUBLIC: &str = "Public";

/// Minimum amount of time that must pass before the index can be rebalanced again
const KEY_REBALANCE_THRESHOLD: &str = "RebalanceThreshold";

/// The ts when the index was last rebalanced
const KEY_LAST_REBALANCE_TS: &str = "LastRebalanceTs";

/// The ts when the index was last updated (any property)
const KEY_LAST_UPDATE_TS: &str = "LastUpdatedTs";

const KEY_TRADE_FEE_TIERS: &str = "TradeFeeTiers";

const KEY_TOTAL_MINTS: &str = "TotalMints";
const KEY_TOTAL_REDEMPTIONS: &str = "TotalRedemptions";

/// Vec<Address> - list of all component addresses
const KEY_COMPONENT_REGISTRY: &str = "ComponentRegistry";

/// Composite key for `(pair, user)` LP share balances.
///
/// Stored under [`TreasuryIndexFundDataKey::UserShares`].
#[contracttype]
#[derive(Clone)]
pub struct UserMonthlyVolumeKey {
    pub user: Address,
    pub month_bucket: u64,
}

/// Persistent storage keys for all per-pair state.
///
/// Everything here is stored in **persistent** storage and must be TTL-bumped
/// (`bump_persistent`) on read/write to avoid expiry.
#[derive(Clone)]
#[contracttype]
enum IndexFundDataKey {
    /// Map of token address to Component
    Component(Address),
    ///
    ComponentBalance(Address),
    /// List of accounts explicitly allowed to mint the index
    Whitelist(Address),
    /// List of accounts blocked from minting the index
    Blacklist(Address),
    /// user + month bucket -> volume
    UserMonthlyVolume(UserMonthlyVolumeKey),
    /// token -> amount
    AccruedProtocolFee(Address),
    /// token -> amount
    AccruedManagerFee(Address),
}

/********** Storage **********/

generate_instance_storage_getter_and_setter!(factory, KEY_FACTORY, Address);
generate_instance_storage_getter_and_setter!(adapter_registry, KEY_ADAPTER_REGISTRY, Address);
generate_instance_storage_getter_and_setter!(token_quote, KEY_TOKEN_QUOTE, Address);
generate_instance_storage_getter_and_setter_with_default!(
    initial_price,
    KEY_INITIAL_PRICE,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(public, KEY_PUBLIC, bool, false);
generate_instance_storage_getter_and_setter_with_default!(
    rebalance_threshold,
    KEY_REBALANCE_THRESHOLD,
    u64,
    THIRTY_DAY
);
generate_instance_storage_getter_and_setter_with_default!(
    last_rebalance_ts,
    KEY_LAST_REBALANCE_TS,
    u64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    last_updated_ts,
    KEY_LAST_UPDATE_TS,
    u64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(total_mints, KEY_TOTAL_MINTS, u128, 0);
generate_instance_storage_getter_and_setter_with_default!(
    total_redemptions,
    KEY_TOTAL_REDEMPTIONS,
    u128,
    0
);

// Component registry management functions

pub fn get_component_registry(e: &Env) -> Vec<Address> {
    // let key = IndexFundDataKey::ComponentRegistry;
    match e.storage().instance().get(&KEY_COMPONENT_REGISTRY) {
        Some(registry) => {
            bump_instance(e);
            registry
        }
        None => Vec::new(e),
    }
}

pub fn add_component_to_registry(env: &Env, token: Address) {
    // let key = IndexFundDataKey::ComponentRegistry;
    let mut registry: Vec<Address> = match env.storage().instance().get(&KEY_COMPONENT_REGISTRY) {
        Some(reg) => reg,
        None => Vec::new(env),
    };

    // Check if component is already in registry
    let len = registry.len();
    for i in 0..len {
        let existing_token = registry.get_unchecked(i);
        if existing_token == token {
            return; // Already exists, don't add duplicate
        }
    }

    // Add new component to registry
    registry.push_back(token);
    env.storage()
        .instance()
        .set(&KEY_COMPONENT_REGISTRY, &registry);
    bump_instance(env);
}

pub fn remove_component_from_registry(env: &Env, token: Address) {
    // let key = IndexFundDataKey::ComponentRegistry;
    let registry: Vec<Address> = match env.storage().instance().get(&KEY_COMPONENT_REGISTRY) {
        Some(reg) => reg,
        None => {
            return;
        } // No registry exists
    };

    // Find and remove the component
    let mut new_registry = Vec::new(env);
    let len = registry.len();
    for i in 0..len {
        let existing_token = registry.get_unchecked(i);
        if existing_token != token {
            new_registry.push_back(existing_token);
        }
    }

    env.storage()
        .instance()
        .set(&KEY_COMPONENT_REGISTRY, &new_registry);
    bump_instance(env);
}

pub fn set_trade_fee_tiers(e: &Env, tiers: Vec<VolumeFeeTier>) {
    e.storage().instance().set(&KEY_TRADE_FEE_TIERS, &tiers);
    bump_instance(e);
}

pub fn get_trade_fee_tiers(e: &Env) -> Vec<VolumeFeeTier> {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&KEY_TRADE_FEE_TIERS)
        .unwrap_or_else(|| {
            Vec::from_array(
                e,
                [
                    VolumeFeeTier {
                        min_monthly_volume: 0,
                        protocol_fee_bps: 100,
                        manager_fee_bps: 0,
                    },
                    VolumeFeeTier {
                        min_monthly_volume: 10_000 * 1_0000000,
                        protocol_fee_bps: 80,
                        manager_fee_bps: 0,
                    },
                    VolumeFeeTier {
                        min_monthly_volume: 50_000 * 1_0000000,
                        protocol_fee_bps: 60,
                        manager_fee_bps: 0,
                    },
                    VolumeFeeTier {
                        min_monthly_volume: 100_000 * 1_0000000,
                        protocol_fee_bps: 40,
                        manager_fee_bps: 0,
                    },
                ],
            )
        })
}

/** PERSTISTENT STORAGE */

// Monthly Volume

pub fn get_user_monthly_volume(e: &Env, user: &Address, month_bucket: u64) -> u128 {
    let key = IndexFundDataKey::UserMonthlyVolume(UserMonthlyVolumeKey {
        user: user.clone(),
        month_bucket,
    });
    match e.storage().persistent().get::<IndexFundDataKey, u128>(&key) {
        Some(volume) => {
            bump_persistent(e, &key);
            volume
        }
        None => 0,
    }
}

pub fn add_user_monthly_volume(e: &Env, user: &Address, month_bucket: u64, amount: u128) {
    let key = IndexFundDataKey::UserMonthlyVolume(UserMonthlyVolumeKey {
        user: user.clone(),
        month_bucket,
    });
    let current = get_user_monthly_volume(e, user, month_bucket);
    let updated = current.saturating_add(amount);
    e.storage().persistent().set(&key, &updated);
    bump_persistent(e, &key);
}

// Fees

pub fn get_accrued_manager_fee(e: &Env, token: Address) -> u128 {
    let key = IndexFundDataKey::AccruedManagerFee(token);
    match e.storage().persistent().get::<IndexFundDataKey, u128>(&key) {
        Some(amount) => {
            bump_persistent(e, &key);
            amount
        }
        None => 0,
    }
}

pub fn set_accrued_manager_fee(e: &Env, token: Address, amount: u128) {
    let key = IndexFundDataKey::AccruedManagerFee(token);
    e.storage().persistent().set(&key, &amount);
    bump_persistent(e, &key);
}

pub fn get_accrued_protocol_fee(e: &Env, token: Address) -> u128 {
    let key = IndexFundDataKey::AccruedProtocolFee(token);
    match e.storage().persistent().get::<IndexFundDataKey, u128>(&key) {
        Some(amount) => {
            bump_persistent(e, &key);
            amount
        }
        None => 0,
    }
}

pub fn set_accrued_protocol_fee(e: &Env, token: Address, amount: u128) {
    let key = IndexFundDataKey::AccruedProtocolFee(token);
    e.storage().persistent().set(&key, &amount);
    bump_persistent(e, &key);
}

// Whitelist/Blacklist functions
// Note: These use manual implementation (not macros) because they are keyed storage patterns
// that require persistent storage, custom TTL management, and Address-based keys.
// This follows the same pattern as Component(Address) and ComponentBalance(Address) storage.

/// Checks if an address is whitelisted
/// Returns true if whitelisted, false if not (missing entries are treated as not whitelisted)
pub fn get_whitelist_status(e: &Env, address: &Address) -> bool {
    let key = IndexFundDataKey::Whitelist(address.clone());
    match e
        .storage()
        .persistent()
        .get::<IndexFundDataKey, Address>(&key)
    {
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
    let key = IndexFundDataKey::Whitelist(address.clone());
    if status {
        e.storage().persistent().set(&key, address);
        bump_persistent(e, &key);
    } else {
        e.storage().persistent().remove(&key);
    }
}

/// Checks if an address is blacklisted
/// Returns true if blacklisted, false if not (missing entries are treated as not blacklisted)
pub fn get_blacklist_status(e: &Env, address: &Address) -> bool {
    let key = IndexFundDataKey::Blacklist(address.clone());
    match e
        .storage()
        .persistent()
        .get::<IndexFundDataKey, Address>(&key)
    {
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
    let key = IndexFundDataKey::Blacklist(address.clone());
    if status {
        e.storage().persistent().set(&key, address);
        bump_persistent(e, &key);
    } else {
        e.storage().persistent().remove(&key);
    }
}

// Component Balance

pub fn get_component_balance(e: &Env, token: Address) -> u128 {
    let key = IndexFundDataKey::ComponentBalance(token);
    match e.storage().persistent().get::<IndexFundDataKey, u128>(&key) {
        Some(balance) => {
            bump_persistent(e, &key);
            balance
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

// Component
pub fn get_all_components(e: &Env) -> Map<Address, Component> {
    let mut components_map = Map::new(e);

    // Get the list of component addresses from registry
    let component_addresses = get_component_registry(e);

    // Iterate through each component address and get its data using index-based access
    let len = component_addresses.len();
    for i in 0..len {
        let address = component_addresses.get_unchecked(i);
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

// Helper function to get component without panicking
pub fn get_component_safe(e: &Env, token: Address) -> Option<Component> {
    let key = IndexFundDataKey::Component(token);
    match e
        .storage()
        .persistent()
        .get::<IndexFundDataKey, Component>(&key)
    {
        Some(component) => {
            bump_persistent(e, &key);
            Some(component)
        }
        None => None,
    }
}

pub fn get_component(e: &Env, token: Address) -> Component {
    let key = IndexFundDataKey::Component(token.clone());
    match e
        .storage()
        .persistent()
        .get::<IndexFundDataKey, Component>(&key)
    {
        Some(component) => {
            bump_persistent(e, &key);
            component
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn set_component(env: &Env, token: Address, component: Component) {
    let key = IndexFundDataKey::Component(token.clone());
    env.storage().persistent().set(&key, &component);
    env.storage().persistent().extend_ttl(&key, 100000, 100000);
}

pub fn remove_component(env: &Env, token: Address) {
    let key = IndexFundDataKey::Component(token.clone());
    env.storage().persistent().remove(&key);

    remove_component_from_registry(env, token.clone());

    let balance_key = IndexFundDataKey::ComponentBalance(token);
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
    let len = component_addresses.len();
    for i in 0..len {
        let address = component_addresses.get_unchecked(i);
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
    let key = IndexFundDataKey::ComponentBalance(token.clone());
    match e.storage().persistent().get::<IndexFundDataKey, u128>(&key) {
        Some(balance) => {
            bump_persistent(e, &key);
            Some(balance)
        }
        None => None,
    }
}

pub fn set_component_balance(env: &Env, token: Address, balance: u128) {
    let key = IndexFundDataKey::ComponentBalance(token);
    env.storage().persistent().set(&key, &balance);
    env.storage().persistent().extend_ttl(&key, 100000, 100000);
}
