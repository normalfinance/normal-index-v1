use soroban_sdk::{contracttype, Address, String, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub struct PrivilegedAddresses {
    pub emergency_admin: Address,
    pub rewards_admin: Address,
    pub operations_admin: Address,
    pub pause_admin: Address,
    pub emergency_pause_admins: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexParams {
    // Address
    pub admin: Address,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    pub normal: bool,    // Whether or not the asset is a Normal Token
    pub asset: Symbol,   // The ticker of the asset
    pub weight: u128,    // The asset's index % allocation (in basis points)
    pub oracle: Address, // The address of the oracle for this asset
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
