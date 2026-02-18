use soroban_sdk::{contracttype, Address, String, Vec};

use crate::component::ComponentUpdate;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexFundAuthorities {
    pub admin: Address,
    pub emergency_admin: Address,
    pub fee_admin: Address,
    pub rewards_admin: Address,
    pub operations_admin: Address,
    pub rebalance_authorities: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexParams {
    pub authorities: IndexFundAuthorities,

    pub token_quote: Address, // usually USDC

    // Config
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub is_public: bool,

    // Price
    pub initial_price: u128,

    // Assets
    pub components: Vec<ComponentUpdate>,
}

// Query Data Structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexFundInfo {
    pub address: Address,
    pub admin_address: Address,
    pub token_address: Address,
    pub total_shares: u128,
    pub initial_price: u128,
    pub is_public: bool,
    pub rebalance_threshold: u64,
    pub last_rebalance_ts: u64,
    pub last_updated_ts: u64,
    pub total_mints: u128,
    pub total_redemptions: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexFundMetrics {
    pub total_shares: u128,
    pub total_mints: u128,
    pub total_redemptions: u128,
    pub current_nav: u128,
    pub share_price: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexFundStatus {
    pub is_public: bool,
    pub can_rebalance: bool,
    pub last_rebalance_ts: u64,
    pub rebalance_threshold: u64,
}
