use soroban_sdk::{contracttype, Address, String, Symbol, Vec};

use crate::adapter::AdapterType;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    pub asset: Symbol,   // The ticker of the asset
    pub weight: u128,    // The asset's index % allocation (in basis points)
    pub oracle: Address, // The address of the oracle for this asset
    pub adapter_type: AdapterType,
    pub adapter: Address,
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
    pub oracle: Option<Address>, // Required for Add, optional for UpdateWeight
    pub adapter_type: AdapterType,
    pub adapter: Address,
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
    pub rebalance_authorities: Vec<Address>,
}
