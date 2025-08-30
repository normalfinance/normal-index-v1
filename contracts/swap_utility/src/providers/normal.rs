use soroban_sdk::{Address, Env, IntoVal, Symbol, Vec};

use normal_rust_types::{SwapError, ProviderConfig, SwapParams, SwapResult};
use crate::providers::base::SwapProvider;

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

        // Execute swap through the Normal Pool Router
        let swap_result: Result<u128, soroban_sdk::Error> = env.invoke_contract(
            &config.contract_address,
            &Symbol::new(env, "swap"),
            Vec::from_array(
                env,
                [
                    env.current_contract_address().into_val(env),
                    params.asset.clone().into_val(env),
                    params.direction.clone().into_val(env),
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
        _env: &Env,
        _token_in: &Address,
        _token_out: &Address,
        amount_in: u128,
        config: &ProviderConfig,
    ) -> Result<u128, SwapError> {
        if !config.is_active {
            return Err(SwapError::ProviderNotConfigured);
        }

        // TODO: Implement proper quote estimation for Normal DEX
        // For now, return a simple estimate based on input amount
        // This should be replaced with actual Normal DEX quote logic
        Ok(amount_in * 95 / 100) // Slippage assumed
    }
}
