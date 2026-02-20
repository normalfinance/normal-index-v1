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

/// Instance key for the originating factory contract address.
const KEY_FACTORY: &str = "Factory";
/// Instance key for the adapter registry contract address.
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

/// Instance key for configured volume-based fee tiers.
const KEY_TRADE_FEE_TIERS: &str = "TradeFeeTiers";

/// Instance key for cumulative minted shares.
const KEY_TOTAL_MINTS: &str = "TotalMints";
/// Instance key for cumulative redeemed shares.
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
    /// Map of token address to tracked balance.
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

/// Returns the component registry address list.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
///
/// # Returns
/// - `Vec<Address>`: Registered component token addresses.
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

/// Adds a token to the component registry if it is not already present.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address to add.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
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

/// Removes a token from the component registry.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address to remove.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
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

/// Sets configured volume fee tiers.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `tiers` (`Vec<VolumeFeeTier>`): Fee-tier schedule to persist.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn set_trade_fee_tiers(e: &Env, tiers: Vec<VolumeFeeTier>) {
    e.storage().instance().set(&KEY_TRADE_FEE_TIERS, &tiers);
    bump_instance(e);
}

/// Returns configured volume fee tiers or the default tier schedule.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
///
/// # Returns
/// - `Vec<VolumeFeeTier>`: Configured tiers, or built-in defaults when unset.
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

/// Returns tracked monthly trading volume for a user and month bucket.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `user` (`&Address`): User address.
/// - `month_bucket` (`u64`): Month bucket index.
///
/// # Returns
/// - `u128`: Tracked monthly volume.
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

/// Adds `amount` to tracked monthly user volume using saturating arithmetic.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `user` (`&Address`): User address.
/// - `month_bucket` (`u64`): Month bucket index.
/// - `amount` (`u128`): Additional volume to add.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
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

/// Returns manager fees accrued for a token.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `token` (`Address`): Token address keyed for fee accrual.
///
/// # Returns
/// - `u128`: Accrued manager fee amount.
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

/// Sets manager fees accrued for a token.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `token` (`Address`): Token address keyed for fee accrual.
/// - `amount` (`u128`): New accrued manager fee amount.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn set_accrued_manager_fee(e: &Env, token: Address, amount: u128) {
    let key = IndexFundDataKey::AccruedManagerFee(token);
    e.storage().persistent().set(&key, &amount);
    bump_persistent(e, &key);
}

/// Returns protocol fees accrued for a token.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `token` (`Address`): Token address keyed for fee accrual.
///
/// # Returns
/// - `u128`: Accrued protocol fee amount.
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

/// Sets protocol fees accrued for a token.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `token` (`Address`): Token address keyed for fee accrual.
/// - `amount` (`u128`): New accrued protocol fee amount.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
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
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `address` (`&Address`): Address to check.
///
/// # Returns
/// - `bool`: `true` if the address is whitelisted.
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
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `address` (`&Address`): Address to update.
/// - `status` (`bool`): Target whitelist status.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
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
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `address` (`&Address`): Address to check.
///
/// # Returns
/// - `bool`: `true` if the address is blacklisted.
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
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `address` (`&Address`): Address to update.
/// - `status` (`bool`): Target blacklist status.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn set_blacklist_status(e: &Env, address: &Address, status: bool) {
    let key = IndexFundDataKey::Blacklist(address.clone());
    if status {
        e.storage().persistent().set(&key, address);
        bump_persistent(e, &key);
    } else {
        e.storage().persistent().remove(&key);
    }
}

/// Returns stored balance for a component token, or panics if uninitialized.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address.
///
/// # Returns
/// - `u128`: Stored component balance.
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

/// Returns all existing components keyed by token address.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
///
/// # Returns
/// - `Map<Address, Component>`: Component metadata by token.
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

/// Returns component metadata for a token, or `None` when missing.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address.
///
/// # Returns
/// - `Option<Component>`: Component metadata if present.
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

/// Returns component metadata for a token, or panics if uninitialized.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address.
///
/// # Returns
/// - `Component`: Component metadata.
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

/// Stores component metadata for a token and refreshes persistent TTL.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address.
/// - `component` (`Component`): Component metadata to persist.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn set_component(env: &Env, token: Address, component: Component) {
    let key = IndexFundDataKey::Component(token.clone());
    env.storage().persistent().set(&key, &component);
    env.storage().persistent().extend_ttl(&key, 100000, 100000);
}

/// Removes a component and its tracked balance, and updates the registry.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn remove_component(env: &Env, token: Address) {
    let key = IndexFundDataKey::Component(token.clone());
    env.storage().persistent().remove(&key);

    remove_component_from_registry(env, token.clone());

    let balance_key = IndexFundDataKey::ComponentBalance(token);
    env.storage().persistent().remove(&balance_key);
}

/// Updates only the weight field for an existing component.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address.
/// - `new_weight` (`u128`): New component weight in basis points.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn update_component_weight(env: &Env, token: Address, new_weight: u128) {
    let mut component = get_component(env, token.clone());
    component.weight = new_weight;
    set_component(env, token, component);
}

/// Returns balances for all registry components, defaulting missing entries to zero.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
///
/// # Returns
/// - `Map<Address, u128>`: Balances by component token address.
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

/// Returns a component balance if present.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address.
///
/// # Returns
/// - `Option<u128>`: Stored balance if present.
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

/// Stores component balance for a token and refreshes persistent TTL.
///
/// # Arguments
/// - `env` (`&Env`): Soroban environment.
/// - `token` (`Address`): Component token address.
/// - `balance` (`u128`): New component balance.
///
/// # Returns
/// - `()` (unit): No direct value is returned.
pub fn set_component_balance(env: &Env, token: Address, balance: u128) {
    let key = IndexFundDataKey::ComponentBalance(token);
    env.storage().persistent().set(&key, &balance);
    env.storage().persistent().extend_ttl(&key, 100000, 100000);
}
