use soroban_sdk::{Address, BytesN, Env, IntoVal, Map, Symbol, Vec};

use crate::{
    errors::SwapError,
    interface::{DexProvider, ProviderConfig, SwapParams, SwapResult},
    providers::base::{build_simple_path, SwapProvider},
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

        // Build sorted tokens vector for pool lookup
        let tokens = build_simple_path(env, &params.token_in, &params.token_out);

        // Get available pools for this token pair
        let pools_result = env.try_invoke_contract::<Map<BytesN<32>, Address>, SwapError>(
            &config.contract_address,
            &Symbol::new(env, "get_pools"),
            Vec::from_array(env, [tokens.clone().into_val(env)]),
        );

        let pool_index = match pools_result {
            Ok(Ok(pools)) => match pools.iter().next() {
                Some((index, _)) => index,
                None => return Err(SwapError::NormalDexFailed),
            },
            _ => return Err(SwapError::NormalDexFailed),
        };

        // Execute swap through the Normal Pool Router
        // swap(user, tokens, token_in, token_out, pool_index, in_amount, out_min) -> u128
        let swap_result: Result<u128, soroban_sdk::Error> = env.invoke_contract(
            &config.contract_address,
            &Symbol::new(env, "swap"),
            Vec::from_array(
                env,
                [
                    params.to.clone().into_val(env),
                    tokens.into_val(env),
                    params.token_in.clone().into_val(env),
                    params.token_out.clone().into_val(env),
                    pool_index.into_val(env),
                    params.amount_in.into_val(env),
                    params.amount_out_min.into_val(env),
                ],
            ),
        );

        match swap_result {
            Ok(amount_out) => Ok(SwapResult {
                provider_used: DexProvider::Normal,
                amount_in: params.amount_in,
                amount_out,
                success: true,
            }),
            Err(_) => Err(SwapError::NormalDexFailed),
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

        // Build sorted tokens vector for pool lookup
        let tokens = build_simple_path(env, token_in, token_out);

        // First, get available pools for this token pair
        let pools_result = env.try_invoke_contract::<Map<BytesN<32>, Address>, SwapError>(
            &config.contract_address,
            &Symbol::new(env, "get_pools"),
            Vec::from_array(env, [tokens.clone().into_val(env)]),
        );

        if let Ok(Ok(pools)) = pools_result {
            // Use the first available pool
            if let Some((pool_index, _)) = pools.iter().next() {
                // Call estimate_swap with correct parameters:
                // estimate_swap(tokens, token_in, token_out, pool_index, in_amount) -> u128
                let quote_result = env.try_invoke_contract::<u128, SwapError>(
                    &config.contract_address,
                    &Symbol::new(env, "estimate_swap"),
                    Vec::from_array(
                        env,
                        [
                            tokens.into_val(env),
                            token_in.clone().into_val(env),
                            token_out.clone().into_val(env),
                            pool_index.into_val(env),
                            amount_in.into_val(env),
                        ],
                    ),
                );

                if let Ok(Ok(amount_out)) = quote_result {
                    return Ok(amount_out);
                }
            }
        }

        // Fallback: estimate with 5% slippage if pool lookup or estimate fails
        Ok(amount_in * 95 / 100)
    }
}
