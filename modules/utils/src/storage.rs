use soroban_sdk::{contracttype, Address, String, Vec};

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
    pub admin: Address,                      // aka manager
    pub rebalance_authorities: Vec<Address>, // New parameter for private index authorities
    pub whitelist_accounts: Vec<Address>,
    pub blacklist_accounts: Vec<Address>,

    // Config
    pub is_public: bool,

    // Token
    pub name: String,
    pub token_symbol: String,
    pub description: String,

    // Price
    pub base_nav: u128,
    pub initial_price: u128,
    pub initial_deposit: u128,

    // Fees
    pub manager_fee_amount: u128,

    // Assets
    pub components: Vec<Address>,
}
