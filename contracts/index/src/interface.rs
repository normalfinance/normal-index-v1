use soroban_sdk::{ contracttype, Address, Env, Map };

use crate::stake::Stake;
use crate::storage::Component;

pub trait IndexTrait {
    // fn get_total_shares(e: Env) -> u128;

    //
    fn mint(
        e: Env,
        user: Address,
        token: Address,
        amount: u128,
        destination: Option<Address>,
        max_slippage: Option<u64>
    );

    //
    fn redeem(e: Env, user: Address, share_amount: u128);
}

pub trait AdminInterface {
    //
    fn initialize(e: Env, admin: Address, token: Address);

    // Set unstaking period
    fn set_unstaking_period(e: Env, admin: Address, unstaking_period: u64);

    // Set max insurance
    fn set_max_shares(e: Env, admin: Address, max_shares: u128);

    fn rebalance(e: Env, admin: Address);

    // Stop staking instantly
    fn kill_mint(e: Env, admin: Address);
    fn kill_redeem(e: Env, admin: Address);
    fn kill_rebalance(e: Env, admin: Address);

    // Resume staking
    fn unkill_mint(e: Env, admin: Address);
    fn unkill_redeem(e: Env, admin: Address);
    fn unkill_rebalance(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_mint(e: Env) -> bool;
    fn get_is_killed_redeem(e: Env) -> bool;
    fn get_is_killed_rebalance(e: Env) -> bool;

    fn set_manager_address(e: Env, admin: Address, manager: Address);
    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address);
    fn distribute_manager_fees(e: Env, admin: Address);
    fn distribute_protocol_fees(e: Env, admin: Address);
    
    // Revenue Share Getters
    fn get_accumulated_manager_fees(e: Env) -> u128;
    fn get_accumulated_protocol_fees(e: Env) -> u128;
    fn get_manager_address(e: Env) -> Address;
    fn get_protocol_fee_recipient(e: Env) -> Address;
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
    pub manager_fee_fraction: u32,
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

// Query Interface
pub trait QueryInterface {
    // Comprehensive index information
    fn get_index_info(e: Env) -> IndexInfo;
    
    // Component and balance queries
    fn get_all_components(e: Env) -> Map<Address, Component>;
    fn get_component_info(e: Env, token: Address) -> Component;
    fn get_all_component_balances(e: Env) -> Map<Address, u128>;
    fn get_component_balance(e: Env, token: Address) -> u128;
    fn get_total_index_value(e: Env) -> u128;
    
    // Financial metrics
    fn get_index_metrics(e: Env) -> IndexMetrics;
    fn get_share_price(e: Env) -> u128;
    fn get_current_nav(e: Env) -> u128;
    
    // Operational status
    fn get_index_status(e: Env) -> IndexStatus;
    fn can_rebalance(e: Env) -> bool;
}
