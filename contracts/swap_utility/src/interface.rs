use soroban_sdk::{contracttype, Address, Env, String, Symbol, Vec};

use crate::errors::SwapError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexDistribution {
    pub protocol_id: String,
    pub path: Vec<Address>,
    pub parts: u32,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapParams {
    pub provider: Option<DexProvider>,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: u128,
    pub amount_out_min: u128,
    pub to: Address,
    pub asset: Symbol,
    pub fee_enabled: Option<bool>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapResult {
    pub provider_used: DexProvider,
    pub amount_in: u128,
    pub amount_out: u128,
    pub success: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderConfig {
    pub contract_address: Address,
    pub is_active: bool,
    pub max_slippage: u64, // in basis points (100 = 1%)
}

pub trait SwapUtilityTrait {
    /// Execute a single swap operation
    fn execute_swap(env: Env, params: SwapParams) -> Result<SwapResult, SwapError>;

    /// Execute multiple swaps in batch
    fn execute_batch_swaps(env: Env, swaps: Vec<SwapParams>) -> Vec<Result<SwapResult, SwapError>>;

    /// Set configuration for a specific provider
    fn set_provider_config(env: Env, admin: Address, provider: DexProvider, config: ProviderConfig);

    /// Get configuration for a specific provider
    fn get_provider_config(env: Env, provider: DexProvider) -> Option<ProviderConfig>;

    /// Initialize the contract
    fn initialize(
        env: Env,
        admin: Address,
        normal_dex_address: Address,
        soroswap_address: Address,
        xlm_token_address: Address,
    );

    /// Check if contract is initialized
    fn is_initialized(env: Env) -> bool;
}
