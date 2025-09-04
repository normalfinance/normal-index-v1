use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

use crate::events::{Events, SwapEvents};
use crate::interface::SwapUtilityTrait;
use crate::providers::{NormalProvider, SoroswapProvider, SwapProvider};
use crate::storage::{
    get_default_provider, get_provider_config, get_xlm_token_address, is_initialized,
    require_admin, require_initialized, set_admin, set_default_provider, set_initialized,
    set_provider_config, set_xlm_token_address,
};
use normal_rust_types::{
    DexProvider, ProviderConfig, SwapDirection, SwapError, SwapParams, SwapResult,
};

#[contract]
pub struct SwapUtility;

#[contractimpl]
impl SwapUtilityTrait for SwapUtility {
    fn initialize(
        env: Env,
        admin: Address,
        normal_dex_address: Address,
        soroswap_address: Address,
        xlm_token_address: Address,
    ) {
        admin.require_auth();

        if is_initialized(&env) {
            panic!("Contract already initialized");
        }

        // Set admin
        set_admin(&env, &admin);

        // Configure Normal DEX provider (active by default)
        let normal_config = ProviderConfig {
            contract_address: normal_dex_address,
            is_active: true,
            max_slippage: 1000, // 10% default max slippage
        };
        set_provider_config(&env, DexProvider::Normal, &normal_config);

        // Configure Soroswap provider (active by default)
        let soroswap_config = ProviderConfig {
            contract_address: soroswap_address,
            is_active: true,
            max_slippage: 500, // 5% default max slippage for Soroswap
        };
        set_provider_config(&env, DexProvider::Soroswap, &soroswap_config);

        // Set Normal as default provider
        set_default_provider(&env, DexProvider::Normal);

        // Store XLM token address for identification
        set_xlm_token_address(&env, &xlm_token_address);

        // Mark as initialized
        set_initialized(&env);
    }

    fn is_initialized(env: Env) -> bool {
        is_initialized(&env)
    }

    fn execute_swap(env: Env, params: SwapParams) -> Result<SwapResult, SwapError> {
        require_initialized(&env);

        // Use the specified provider
        let provider = params.provider.clone();

        // Get provider configuration
        let config =
            get_provider_config(&env, provider.clone()).ok_or(SwapError::ProviderNotConfigured)?;

        if !config.is_active {
            return Err(SwapError::ProviderNotConfigured);
        }

        // Execute swap based on provider with fallback mechanism
        let result = execute_swap_with_fallback(&env, &params, provider, &config);

        // Emit events based on result
        let events = Events::new(&env);
        match &result {
            Ok(swap_result) => {
                events.swap_executed(
                    swap_result.provider_used.clone(),
                    params.token_in.clone(),
                    params.token_out.clone(),
                    swap_result.amount_in,
                    swap_result.amount_out,
                    params.to.clone(),
                );
            }
            Err(error) => {
                let provider_used = params.provider.clone();
                events.swap_failed(
                    provider_used,
                    params.token_in,
                    params.token_out,
                    params.amount_in,
                    *error as u32,
                );
            }
        }

        result
    }

    fn execute_batch_swaps(env: Env, swaps: Vec<SwapParams>) -> Vec<Result<SwapResult, SwapError>> {
        require_initialized(&env);

        let mut results: Vec<Result<SwapResult, SwapError>> = Vec::new(&env);

        for i in 0..swaps.len() {
            let swap = swaps.get(i).unwrap();
            let result = Self::execute_swap(env.clone(), swap);
            results.push_back(result);
        }

        results
    }

    fn set_provider_config(
        env: Env,
        admin: Address,
        provider: DexProvider,
        config: ProviderConfig,
    ) {
        admin.require_auth();
        require_admin(&env, &admin);

        set_provider_config(&env, provider.clone(), &config);

        let events = Events::new(&env);
        events.provider_config_set(provider, config.contract_address, admin);
    }

    fn get_provider_config(env: Env, provider: DexProvider) -> Option<ProviderConfig> {
        get_provider_config(&env, provider)
    }
}

// Helper function to execute swap with fallback mechanism
fn execute_swap_with_fallback(
    env: &Env,
    params: &SwapParams,
    primary_provider: DexProvider,
    primary_config: &ProviderConfig,
) -> Result<SwapResult, SwapError> {
    // Try primary provider first
    let primary_result = match primary_provider {
        DexProvider::Normal => NormalProvider::execute_swap(env, params, primary_config),
        DexProvider::Soroswap => SoroswapProvider::execute_swap(env, params, primary_config),
    };

    match primary_result {
        Ok(result) => Ok(result),
        Err(SwapError::InsufficientLiquidity) | Err(SwapError::SoroswapSwapFailed) => {
            // Try fallback to Normal DEX if Soroswap fails, but only for Normal tokens
            if matches!(primary_provider, DexProvider::Soroswap)
                && is_normal_token(env, &params.asset)
            {
                if let Some(fallback_config) = get_provider_config(env, DexProvider::Normal) {
                    if fallback_config.is_active {
                        return NormalProvider::execute_swap(env, params, &fallback_config);
                    }
                }
            }
            primary_result
        }
        Err(other_error) => Err(other_error),
    }
}

// Additional helper functions
impl SwapUtility {
    pub fn get_best_quote(
        env: Env,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        asset: Symbol,
    ) -> Result<(DexProvider, u128), SwapError> {
        require_initialized(&env);

        let params = SwapParams {
            provider: get_default_provider(&env),
            token_in: token_in.clone(),
            token_out: token_out.clone(),
            amount_in,
            amount_out_min: 0,
            to: env.current_contract_address(), // placeholder
            asset: asset.clone(),
            direction: SwapDirection::Buy,
            fee_enabled: None,
        };

        let selected_provider = select_provider(&env, &params);

        match selected_provider {
            DexProvider::Normal => {
                if let Some(normal_config) = get_provider_config(&env, DexProvider::Normal) {
                    if normal_config.is_active {
                        if let Ok(amount) = NormalProvider::get_estimated_output(
                            &env,
                            &token_in,
                            &token_out,
                            amount_in,
                            &normal_config,
                        ) {
                            return Ok((DexProvider::Normal, amount));
                        }
                    }
                }
                Err(SwapError::ProviderNotConfigured)
            }
            DexProvider::Soroswap => Err(SwapError::SoroswapAggregatorUnavailable),
        }
    }

    /// Set a provider as active or inactive
    pub fn set_provider_active(
        env: Env,
        admin: Address,
        provider: DexProvider,
        is_active: bool,
    ) -> Result<(), SwapError> {
        admin.require_auth();
        require_admin(&env, &admin);

        if let Some(mut config) = get_provider_config(&env, provider.clone()) {
            config.is_active = is_active;
            set_provider_config(&env, provider, &config);
            Ok(())
        } else {
            Err(SwapError::ProviderNotConfigured)
        }
    }
}

// Token identification utilities
fn is_normal_token(env: &Env, symbol: &Symbol) -> bool {
    let normal_prefixes = [
        Symbol::new(&env, "nUSDC"),
        Symbol::new(&env, "nUSDT"),
        Symbol::new(&env, "nBTC"),
        Symbol::new(&env, "nETH"),
        Symbol::new(&env, "nXLM"),
    ];

    for normal_symbol in normal_prefixes.iter() {
        if symbol == normal_symbol {
            return true;
        }
    }

    false
}

fn is_xlm_token(env: &Env, address: &Address) -> bool {
    match get_xlm_token_address(env) {
        Some(xlm_address) => *address == xlm_address,
        None => false,
    }
}

fn select_provider(env: &Env, params: &SwapParams) -> DexProvider {
    let is_normal = is_normal_token(env, &params.asset);

    if !is_normal {
        return DexProvider::Soroswap;
    }

    let trading_with_xlm =
        is_xlm_token(env, &params.token_in) || is_xlm_token(env, &params.token_out);

    if trading_with_xlm {
        DexProvider::Normal
    } else {
        DexProvider::Soroswap
    }
}
