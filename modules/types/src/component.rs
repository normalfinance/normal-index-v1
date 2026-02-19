use soroban_sdk::{contracttype, Address, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    pub asset: Symbol,   // The ticker of the asset
    pub weight: u128,    // The asset's index % allocation (in basis points)
    pub oracle: Address, // The address of the oracle for this asset
    pub adapter: Symbol,
}

// Rebalancing Data Structures
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComponentAction {
    Add,
    Remove,
    UpdateWeight,
    UpdateOracle,
    UpdateAdapter,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentUpdate {
    pub token: Address,
    pub action: ComponentAction,
    pub new_weight: Option<u128>,
    pub new_oracle: Option<Address>,
    pub new_adapter: Option<Symbol>,
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
