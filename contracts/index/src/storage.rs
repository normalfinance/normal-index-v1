use paste::paste;
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{ contracttype, panic_with_error, Address, Env, Map, Symbol };
use utils::bump::{ bump_instance, bump_persistent };
use utils::constant::THIRTY_DAY;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default,
    generate_instance_storage_setter,
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

    Whitelist(Address), // List of accounts explicitly allowed to mint the index
    Blacklist(Address), // List of accounts blocked from minting the index

    RebalanceThreshold, // Minimum amount of time that must pass before the index can be rebalanced again

    LastRebalanceTs, // The ts when the index was last rebalanced
    LastUpdatedTs, // The ts when the index was last updated (any property)

    // Metrics
    TotalMints,
    TotalRedemptions,
    TotalFees,

    // Paused operations
    IsKilledMint,
    IsKilledRedeem,
    IsKilledRebalance,
}

generate_instance_storage_getter_and_setter_with_default!(factory, DataKey::Factory, Address, Address::from_str(&Env::default(), ""));

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
generate_instance_storage_getter_and_setter_with_default!(public, DataKey::Public, bool, false);
generate_instance_storage_getter_and_setter_with_default!(
    rebalance_threshold,
    DataKey::RebalanceThreshold,
    u64,
    THIRTY_DAY
);

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
    // This function needs to be implemented properly to iterate through all components
    // For now, return an empty map as placeholder
    Map::new(e)
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

fn set_component(env: &Env, token: Address, component: Component) {
    let key = DataKey::Component(token.clone());
    env.storage().persistent().set(&key, &component);
    env.storage().persistent().extend_ttl(&key, 100000, 100000);
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
    e.storage().instance().get(&DataKey::TotalShares).unwrap_or(0)
}

pub fn set_max_shares(e: &Env, max_shares: &u128) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::TotalShares, max_shares);
}

pub fn get_unstaking_period(e: &Env) -> u64 {
    bump_instance(e);
    e.storage().instance().get(&DataKey::LastRebalanceTs).unwrap_or(0)
}

pub fn set_unstaking_period(e: &Env, period: &u64) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::LastRebalanceTs, period);
}

pub fn get_shares_base(e: &Env) -> u128 {
    bump_instance(e);
    e.storage().instance().get(&DataKey::TotalShares).unwrap_or(0)
}


