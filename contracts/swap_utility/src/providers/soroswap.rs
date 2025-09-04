use soroban_sdk::{Address, BytesN, Env, IntoVal, Symbol, Vec};

use crate::providers::base::{build_simple_path, SwapProvider};
use normal_rust_types::{DexProvider, ProviderConfig, SwapError, SwapParams, SwapResult};

pub struct SoroswapProvider;

impl SwapProvider for SoroswapProvider {
    fn execute_swap(
        env: &Env,
        params: &SwapParams,
        config: &ProviderConfig,
    ) -> Result<SwapResult, SwapError> {
        // Validate parameters
        Self::validate_params(env, params)?;

        if !config.is_active {
            return Err(SwapError::ProviderNotConfigured);
        }

        // Build swap path for Soroswap
        let path = build_simple_path(env, &params.token_in, &params.token_out);

        // Set deadline to 5 minutes from now if not specified
        let deadline = env.ledger().timestamp() + 300u64;

        // Execute swap through Soroswap aggregator
        // Based on: https://github.com/soroswap/aggregator/blob/83b51cdf26ea56b5688bc7c72f319ef0e854f698/contracts/adapters/interface/src/lib.rs#L33
        let swap_result: Result<Vec<i128>, soroban_sdk::Error> = env.invoke_contract(
            &config.contract_address,
            &Symbol::new(env, "swap_exact_tokens_for_tokens"),
            Vec::from_array(
                env,
                [
                    params.amount_in.into_val(env),
                    params.amount_out_min.into_val(env),
                    path.into_val(env),
                    params.to.clone().into_val(env),
                    deadline.into_val(env),
                    None::<Vec<BytesN<32>>>.into_val(env), // Optional additional swap data
                ],
            ),
        );

        match swap_result {
            Ok(amounts) => {
                // Soroswap returns a vector of amounts for each step in the path
                // For a simple two-token swap, the output amount is the last element
                let amount_out = amounts.last().unwrap_or(0) as u128;

                Ok(SwapResult {
                    provider_used: DexProvider::Soroswap,
                    amount_in: params.amount_in,
                    amount_out,
                    success: true,
                })
            }
            Err(_) => Err(SwapError::SoroswapSwapFailed),
        }
    }

    fn validate_params(_env: &Env, params: &SwapParams) -> Result<(), SwapError> {
        // Check that tokens are different
        if params.token_in == params.token_out {
            return Err(SwapError::InvalidTokenPair);
        }

        // Check that amount is positive
        if params.amount_in <= 0 {
            return Err(SwapError::InvalidAmount);
        }

        // Check that minimum output is non-negative
        if params.amount_out_min < 0 {
            return Err(SwapError::InvalidAmount);
        }

        Ok(())
    }

    fn get_estimated_output(
        env: &Env,
        token_in: &Address,
        token_out: &Address,
        amount_in: u128,
        config: &ProviderConfig,
    ) -> Result<u128, SwapError> {
        if !config.is_active {
            return Err(SwapError::ProviderNotConfigured);
        }

        // Build path for quote
        let path = build_simple_path(env, token_in, token_out);

        // Try to get a quote from Soroswap aggregator
        let quote_result: Result<Vec<i128>, soroban_sdk::Error> = env.invoke_contract(
            &config.contract_address,
            &Symbol::new(env, "get_amounts_out"),
            Vec::from_array(env, [amount_in.into_val(env), path.into_val(env)]),
        );

        match quote_result {
            Ok(amounts) => {
                let estimated_amount = amounts.last().unwrap_or(0) as u128;
                Ok(estimated_amount)
            }
            Err(_) => {
                // Fallback to simple estimation if quote fails
                Ok(amount_in * 95 / 100) // Assume 5% slippage for estimation
            }
        }
    }
}

impl SoroswapProvider {
    /// Helper function to check if Soroswap aggregator is available
    pub fn is_aggregator_available(env: &Env, config: &ProviderConfig) -> bool {
        if !config.is_active {
            return false;
        }

        // Try a simple contract call to check if the aggregator is responsive
        let result: Result<bool, soroban_sdk::Error> = env.invoke_contract(
            &config.contract_address,
            &Symbol::new(env, "initialized"), // Assuming there's a health check function
            Vec::new(env),
        );

        result.is_ok()
    }

    /// Get the optimal path for a swap (could be extended for multi-hop swaps)
    pub fn get_optimal_path(env: &Env, token_in: &Address, token_out: &Address) -> Vec<Address> {
        // For now, return simple direct path
        // Future enhancement: query Soroswap for optimal multi-hop paths
        build_simple_path(env, token_in, token_out)
    }
}
