use soroban_sdk::{contracttype, Address, Env, String, Symbol, Vec};

use normal_rust_types::{SwapError, ProviderConfig};

impl Default for SwapDirection {
    fn default() -> Self {
        Self::Buy
    }
}

impl Default for DexProvider {
    fn default() -> Self {
        Self::Normal
    }
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
