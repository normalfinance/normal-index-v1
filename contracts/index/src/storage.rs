use paste::paste;
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{ contracttype, panic_with_error, Address, Env, Symbol };
use utils::bump::{ bump_instance, bump_persistent, bump_temporary };
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

generate_instance_storage_getter_and_setter!(factory, DataKey::Factory, Address);

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
    last_rebalance_ts,
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
    match e.storage().persistent().get::<DataKey, i128>(&key) {
        Some(component) => {
            bump_persistent(e, &key);
            component
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    };
}

// Component

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    pub asset: Symbol,
    pub weight: u128,
}

pub fn get_all_components(e: &Env) -> Map<Address, Component> {
    let key = DataKey::Component;
    e.storage().persistent().get(&key)
}

pub fn get_component(e: &Env, token: Address) -> Component {
    let key = DataKey::Component(token);
    match e.storage().persistent().get::<DataKey, i128>(&key) {
        Some(component) => {
            bump_persistent(e, &key);
            component
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    };
}

fn set_component(env: &Env, token: Address, amount: i128) {
    let key = DataKey::Component(token.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage().persistent().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    // Updating the <addr> last transfer info for fee tracking
    // let last_transfer = get_last_transfer(env, &addr);
    let updated_last_transfer = LastTransfer {
        ts: env.ledger().timestamp(),
        balance: amount,
    };
    save_last_transfer(env, &addr, &updated_last_transfer);
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
    is_killed_deposit,
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
