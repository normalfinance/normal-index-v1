use soroban_sdk::{contracttype, Address, Env, Map, Vec};

use crate::storage::Component;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DexProvider {
    Normal,
    Soroswap,
}

impl Default for DexProvider {
    fn default() -> Self {
        Self::Normal
    }
}

pub trait IndexTrait {
    fn mint(
        e: Env,
        user: Address,
        token: Address,
        amount: u128,
        destination: Option<Address>,
        max_slippage: Option<u64>,
    );

    fn redeem(e: Env, user: Address, share_amount: u128);

    fn get_token(e: Env) -> Address;

    fn get_factory(e: Env) -> Address;

    fn get_base_nav(e: Env) -> u128;

    fn get_initial_price(e: Env) -> u128;

    fn get_nav(e: Env) -> i128;

    fn get_price(e: Env) -> i128;

    fn get_total_shares(e: Env) -> u128;

    fn get_public_status(e: Env) -> bool;

    fn get_whitelist_status(e: Env, address: Address) -> bool;

    fn get_blacklist_status(e: Env, address: Address) -> bool;

    fn get_manager_fee_amount(e: Env) -> u128;

    fn get_rebalance_threshold(e: Env) -> u64;

    fn get_last_rebalance_timestamp(e: Env) -> u64;

    fn get_last_updated_timestamp(e: Env) -> u64;

    fn get_total_mints(e: Env) -> u128;

    fn get_total_redemptions(e: Env) -> u128;

    fn get_total_fees(e: Env) -> u128;

    fn get_component(e: Env, token: Address) -> crate::storage::Component;

    fn get_component_balance(e: Env, token: Address) -> u128;

    fn get_last_fee_collection(e: Env) -> u64;

    /// Transfer shares between users with proper fee handling
    fn transfer_shares(e: Env, from: Address, to: Address, amount: u128);

    /// Transfer shares from allowance with proper fee handling  
    fn transfer_shares_from(e: Env, spender: Address, from: Address, to: Address, amount: u128);
}

pub trait AdminInterface {
    fn initialize(e: Env, admin: Address, token: Address);

    fn refactor(e: Env, caller: Address, params: RefactorParams);

    fn rebalance(e: Env, caller: Address, params: RebalanceParams);

    fn set_rebalance_authority(e: Env, admin: Address, authority: Address, status: bool);

    fn distribute_manager_fees(e: Env, admin: Address);

    fn distribute_protocol_fees(e: Env, admin: Address);

    fn set_factory(e: Env, admin: Address, factory: Address);

    fn set_base_nav(e: Env, admin: Address, base_nav: u128);

    fn set_initial_price(e: Env, admin: Address, initial_price: u128);

    fn set_public_status(e: Env, admin: Address, public: bool);

    fn set_whitelist_status(e: Env, admin: Address, address: Address, status: bool);

    fn set_blacklist_status(e: Env, admin: Address, address: Address, status: bool);

    fn set_manager_address(e: Env, admin: Address, manager: Address);

    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address);

    fn set_manager_fee_amount(e: Env, admin: Address, fee_amount: u128);

    fn set_rebalance_threshold(e: Env, admin: Address, threshold: u64);

    //    _______     __       ____  ____   ________  _______  ________
    //   |   __ "\   /""\     ("  _||_ " | /"       )/"     "||"      "\
    //   (. |__) :) /    \    |   (  ) : |(:   \___/(: ______)(.  ___  :)
    //   |:  ____/ /' /\  \   (:  |  | . ) \___  \   \/    |  |: \   ) ||
    //   (|  /    //  __'  \   \\ \__/ //   __/  \\  // ___)_ (| (___\ ||
    //  /|__/ \  /   /  \\  \  /\\ __ //\  /" \   :)(:      "||:       :)
    // (_______)(___/    \___)(__________)(_______/  \_______)(________/

    fn kill_mint(e: Env, admin: Address);
    fn kill_redeem(e: Env, admin: Address);
    fn kill_rebalance(e: Env, admin: Address);

    fn unkill_mint(e: Env, admin: Address);
    fn unkill_redeem(e: Env, admin: Address);
    fn unkill_rebalance(e: Env, admin: Address);

    fn get_is_killed_mint(e: Env) -> bool;
    fn get_is_killed_redeem(e: Env) -> bool;
    fn get_is_killed_rebalance(e: Env) -> bool;

    fn get_accumulated_manager_fees(e: Env) -> u128;
    fn get_accumulated_protocol_fees(e: Env) -> u128;
}

// Query Data Structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexInfo {
    pub address: Address,
    pub token_address: Address,
    pub total_shares: u128,
    pub base_nav: u128,
    pub initial_price: u128,
    pub is_public: bool,
    pub manager_fee_amount: u128,
    pub manager_address: Address,
    pub protocol_fee_recipient: Address,
    pub accumulated_manager_fees: u128,
    pub accumulated_protocol_fees: u128,
    pub last_rebalance_ts: u64,
    pub last_updated_ts: u64,
    pub total_mints: u128,
    pub total_redemptions: u128,
    pub total_fees: u128,
    pub is_killed_mint: bool,
    pub is_killed_redeem: bool,
    pub is_killed_rebalance: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexMetrics {
    pub total_shares: u128,
    pub total_mints: u128,
    pub total_redemptions: u128,
    pub total_fees: u128,
    pub accumulated_manager_fees: u128,
    pub accumulated_protocol_fees: u128,
    pub current_nav: u128,
    pub share_price: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexStatus {
    pub is_killed_mint: bool,
    pub is_killed_redeem: bool,
    pub is_killed_rebalance: bool,
    pub is_public: bool,
    pub can_rebalance: bool,
    pub last_rebalance_ts: u64,
    pub rebalance_threshold: u64,
}

// Rebalancing Data Structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComponentAction {
    Add,
    Remove,
    UpdateWeight,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentUpdate {
    pub token: Address,
    pub new_weight: u128,
    pub action: ComponentAction,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefactorParams {
    pub component_updates: Vec<ComponentUpdate>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceParams {
    pub target_nav: Option<i128>, // Optional NAV target for rebalancing
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentAllocation {
    pub component: Component,
    pub current_balance: u128,
    pub target_balance: u128,
    pub percentage_of_nav: u128, // In basis points
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceStatus {
    pub can_rebalance: bool,
    pub time_until_next_rebalance: u64,
    pub last_rebalance_ts: u64,
    pub rebalance_threshold: u64,
    pub is_public: bool,
    pub authorized_rebalancers: Vec<Address>, // For private indexes
}

// Query Interface
pub trait QueryInterface {
    // Comprehensive index information
    fn get_index_info(e: Env) -> IndexInfo;

    // Component and balance queries
    fn get_all_components(e: Env) -> Map<Address, Component>;
    fn get_component_info(e: Env, token: Address) -> Component;
    fn get_all_component_balances(e: Env) -> Map<Address, u128>;
    fn get_total_index_value(e: Env) -> u128;

    // Financial metrics
    fn get_index_metrics(e: Env) -> IndexMetrics;
    fn get_share_price(e: Env) -> u128;
    fn get_current_nav(e: Env) -> u128;

    // Operational status
    fn get_index_status(e: Env) -> IndexStatus;
    fn can_rebalance(e: Env) -> bool;

    // Rebalancing queries
    fn get_rebalance_status(e: Env) -> RebalanceStatus;
    fn can_address_rebalance(e: Env, caller: Address) -> bool;
    fn get_component_allocation(e: Env) -> Map<Address, ComponentAllocation>;
    fn get_rebalance_authorities(e: Env) -> Vec<Address>;
}
