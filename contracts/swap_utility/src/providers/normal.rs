use soroban_sdk::{Address, Env, IntoVal, String, Symbol, Vec};

use crate::{
    errors::SwapError,
    interface::{DexDistribution, DexProvider, ProviderConfig, SwapParams, SwapResult},
    providers::base::SwapProvider,
};

pub struct NormalProvider;

impl SwapProvider for NormalProvider {
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

        // Create distribution for Normal DEX (simple direct swap)
        let distribution = Self::create_distribution(env, &params.token_in, &params.token_out);

        // Set deadline to 5 minutes from now
        let deadline = env.ledger().timestamp() + 300u64;

        // Execute swap through Normal DEX factory using index factory interface
        let swap_result: Result<Vec<Vec<i128>>, soroban_sdk::Error> = env.invoke_contract(
            &config.contract_address,
            &Symbol::new(env, "swap"),
            Vec::from_array(
                env,
                [
                    params.token_in.clone().into_val(env),
                    params.token_out.clone().into_val(env),
                    params.amount_in.into_val(env),
                    params.amount_out_min.into_val(env),
                    distribution.into_val(env),
                    params.to.clone().into_val(env),
                    deadline.into_val(env),
                ],
            ),
        );

        match swap_result {
            Ok(amounts) => {
                // Extract the final output amount from the result
                let amount_out = amounts
                    .last()
                    .and_then(|inner_vec| inner_vec.last())
                    .unwrap_or(0);

                Ok(SwapResult {
                    provider_used: DexProvider::Normal,
                    amount_in: params.amount_in,
                    amount_out,
                    success: true,
                })
            }
            Err(_) => Err(SwapError::NormalDexFailed),
        }
    }

    fn validate_params(env: &Env, params: &SwapParams) -> Result<(), SwapError> {
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

        // For now, return a simple estimate with 2% slippage
        // In the future, this could call a quote function if the factory provides one
        Ok(amount_in * 98 / 100)
    }
}

impl NormalProvider {
    /// Create a simple distribution for direct token swap
    fn create_distribution(
        env: &Env,
        token_in: &Address,
        token_out: &Address,
    ) -> Vec<DexDistribution> {
        let mut distribution = Vec::new(env);

        // Create path for direct swap
        let mut path = Vec::new(env);
        path.push_back(token_in.clone());
        path.push_back(token_out.clone());

        distribution.push_back(DexDistribution {
            protocol_id: String::from_str(env, "normal"),
            path,
            parts: 100, // 100% through this path
        });

        distribution
    }
}
