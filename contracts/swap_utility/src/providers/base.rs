use soroban_sdk::{Address, Env, Vec};

use crate::{
    errors::SwapError,
    interface::{ProviderConfig, SwapParams, SwapResult},
};

pub trait SwapProvider {
    /// Execute a swap operation for this specific provider
    fn execute_swap(
        env: &Env,
        params: &SwapParams,
        config: &ProviderConfig,
    ) -> Result<SwapResult, SwapError>;

    /// Validate that the swap parameters are correct for this provider
    fn validate_params(env: &Env, params: &SwapParams) -> Result<(), SwapError>;

    /// Get an estimated output amount for the given input (optional)
    fn get_estimated_output(
        env: &Env,
        token_in: &Address,
        token_out: &Address,
        amount_in: u128,
        config: &ProviderConfig,
    ) -> Result<u128, SwapError> {
        // Default implementation returns the input amount (1:1 ratio)
        // Providers can override this for better estimation
        Ok(amount_in)
    }
}

/// Helper function to build a simple two-token swap path
pub fn build_simple_path(env: &Env, token_in: &Address, token_out: &Address) -> Vec<Address> {
    let mut path = Vec::new(env);
    path.push_back(token_in.clone());
    path.push_back(token_out.clone());
    path
}
