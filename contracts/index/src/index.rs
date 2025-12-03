use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    contracttype,
    panic_with_error,
    Address,
    Env,
    IntoVal,
    log,
    String,
    Symbol,
    Vec,
};
use token_share::get_token_share;
use utils::validate;

// Types to match the SwapUtility contract
#[derive(Clone)]
#[contracttype]
pub struct SwapUtilityParams {
    pub provider: Option<String>, // DexProvider as string
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
    pub amount_out_min: i128,
    pub to: Address,
    pub asset: Symbol,
    pub direction: SwapDirection,
}

use crate::errors::IndexError;
use crate::events::{ Events, IndexEvents };
use crate::storage::{ get_all_components, get_swap_utility, get_swap_utility_address };

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SwapDirection {
    Buy,
    Sell,
}

impl Default for SwapDirection {
    fn default() -> Self {
        Self::Buy
    }
}

#[contracttype]
pub struct SwapParams {
    pub provider: Option<String>, // DexProvider as string (Option<Enum> not supported by contracttype)
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
    pub amount_out_min: i128,
    pub to: Address,
}

pub fn generate_swap_params(
    e: &Env,
    deposit_token: Address,
    deposit_amount: u128
) -> Vec<SwapParams> {
    // Validate inputs
    validate!(e, deposit_amount > 0, IndexError::PathIsEmpty);

    let components = get_all_components(e);
    let mut swaps: Vec<SwapParams> = Vec::new(e);

    // Return empty if no components are configured
    if components.is_empty() {
        return swaps;
    }

    let total_deposit = deposit_amount;

    for (component_address, component) in components.iter() {
        // Calculate target allocation based on component weight
        // component.weight is in basis points (10000 = 100%)
        let target_allocation = (total_deposit * (component.weight as u128)) / 10000;

        // Only create swap if we need to buy this component and allocation > 0
        if target_allocation > 0 {
            // Validate that deposit token is different from component token
            if deposit_token == component_address {
                continue; // Skip if trying to swap token for itself
            }

            // Calculate minimum output with 5% slippage tolerance
            let min_output = ((target_allocation as i128) * 95) / 100;
            validate!(e, min_output > 0, IndexError::PathIsEmpty);

            //Revisit this param generation here
            let swap = SwapParams {
                provider: None, // Use default provider
                token_in: deposit_token.clone(),
                token_out: component_address.clone(),
                amount_in: target_allocation as i128,
                amount_out_min: min_output,
                to: e.current_contract_address(),
            };

            swaps.push_back(swap);
        }
    }

    swaps
}

pub fn execute_swaps(e: &Env, swaps: Vec<SwapParams>) -> Vec<u128> {
    let mut results: Vec<u128> = Vec::new(e);

    // Get swap utility contract address (stored for potential future use)
    let _swap_utility_address = get_swap_utility_address(e);

    for i in 0..swaps.len() {
        let params = swaps.get(i).unwrap();

        log!(&e, "Executing swap with params: {:?}", params);

        // Get the component info to extract the asset symbol
        // For buy swaps: token_out is the component, for sell swaps: token_in is the component
        let base_token = get_base_token(e);
        let component_token = if params.token_out == base_token {
            // Sell swap: selling component for base token
            params.token_in.clone()
        } else {
            // Buy swap: buying component with base token
            params.token_out.clone()
        };
        let component = crate::storage::get_component(e, component_token);

        log!(&e, "Component in execute_swaps: {:?}", component);

        // Map local SwapParams to SwapUtilityParams for the external contract
        let utility_params = SwapUtilityParams {
            provider: None, // Let the SwapUtility contract decide which provider to use
            token_in: params.token_in.clone(),
            token_out: params.token_out.clone(),
            amount_in: params.amount_in,
            amount_out_min: params.amount_out_min,
            to: params.to.clone(),
            asset: component.asset.clone(),
            direction: SwapDirection::Buy, // We're always buying components
        };

        // Execute individual swap via cross-contract call to swap utility
        let swap_result = e.try_invoke_contract::<SwapResult, soroban_sdk::Error>(
            &get_swap_utility(&e),
            &Symbol::new(&e, "execute_swap"),
            Vec::from_array(&e, [utility_params.into_val(e)])
        );

        match swap_result {
            Ok(Ok(result)) => {
                // Successful swap - SwapResult from utility contract
                Events::new(&e).swap(
                    Vec::new(&e),
                    e.current_contract_address(),
                    result.provider_used,
                    params.token_in,
                    params.token_out,
                    result.amount_in,
                    result.amount_out
                );

                // Add successful result
                results.push_back(result.amount_out as u128);
            }
            Ok(Err(_swap_error)) => {
                // Swap failed but call succeeded - emit failure event
                Events::new(&e).swap_failed(
                    e.current_contract_address(),
                    params.token_in,
                    params.token_out,
                    params.amount_in.try_into().unwrap(),
                    1000u32 // Convert error enum to numeric code
                );

                // Add zero result for failed swap
                results.push_back(0u128);
            }
            Err(_contract_error) => {
                // Contract call failed - emit failure event with generic error
                Events::new(&e).swap_failed(
                    e.current_contract_address(),
                    params.token_in,
                    params.token_out,
                    params.amount_in.try_into().unwrap(),
                    999u32 // Generic contract call failure
                );

                // Add zero result for failed swap
                results.push_back(0u128);
            }
        }
    }

    results
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SwapError {
    ProviderNotSupported = 100,
    ProviderNotConfigured = 101,
    InvalidTokenPair = 200,
    InvalidAmount = 201,
    InvalidSlippage = 202,
    InsufficientLiquidity = 300,
    SlippageExceeded = 301,
    SwapFailed = 302,
    NormalDexFailed = 400,
    SoroswapSwapFailed = 401,
    SoroswapAggregatorUnavailable = 402,
    InvalidProviderConfig = 500,
    UnauthorizedAccess = 501,
    ContractNotInitialized = 502,
}

impl From<soroban_sdk::Error> for SwapError {
    fn from(_: soroban_sdk::Error) -> Self {
        SwapError::SwapFailed
    }
}

pub fn vault_amount_to_shares(
    e: &Env,
    amount: u128,
    total_shares: u128,
    vault_amount: u128
) -> u128 {
    // relative to the entire pool + total amount minted
    let n_shares = if vault_amount > 0 {
        // assumes total_shares != 0 (in most cases) for nice result for user
        amount.fixed_mul_floor(total_shares, vault_amount).unwrap_or(amount)
        // get_proportion_u128(e, amount, total_shares, vault_amount)
    } else {
        // must be case that total_shares == 0 for nice result for user
        validate!(e, total_shares == 0, IndexError::InvalidSharesDetected);

        amount
    };

    n_shares
}

// Enhanced rebalancing swap generation - now focuses on balancing existing components
pub fn generate_rebalance_swaps(
    e: &Env,
    params: &crate::interface::RebalanceParams
) -> Vec<SwapParams> {
    let current_nav = crate::storage::get_base_nav(e) as u128; // Simplified NAV calculation
    let target_nav = params.target_nav.map(|n| n as u128).unwrap_or(current_nav);

    let mut swaps = Vec::new(e);

    // Get all current components and their weights
    let components = crate::storage::get_all_components(e);

    // Get component addresses for iteration
    let component_addresses = crate::storage::get_component_registry(e);

    // For each component, check if current balance matches target allocation
    let len = component_addresses.len();
    for i in 0..len {
        let token_address = component_addresses.get_unchecked(i);
        let component = components.get(token_address.clone()).unwrap();

        let current_balance = crate::storage
            ::get_component_balance_safe(e, token_address.clone())
            .unwrap_or(0);

        // Calculate target balance based on component weight and target NAV
        let target_balance = if target_nav > 0 {
            (target_nav * component.weight) / 10000
        } else {
            0
        };

        if target_balance > current_balance {
            // Need to buy more of this component
            let amount_needed = target_balance - current_balance;
            if amount_needed > 0 {
                let swap = create_buy_swap(e, token_address.clone(), amount_needed);
                swaps.push_back(swap);
            }
        } else if target_balance < current_balance {
            // Need to sell some of this component
            let amount_to_sell = current_balance - target_balance;
            if amount_to_sell > 0 {
                let swap = create_sell_swap(e, token_address.clone(), amount_to_sell);
                swaps.push_back(swap);
            }
        }
        // If target_balance == current_balance, no swap needed for this component
    }

    swaps
}

fn create_buy_swap(e: &Env, token_out: Address, amount_needed: u128) -> SwapParams {
    let base_token = get_base_token(e);

    SwapParams {
        provider: None, // Use default provider
        token_in: base_token,
        token_out,
        amount_in: amount_needed as i128, // Simplified 1:1 ratio
        amount_out_min: ((amount_needed as i128) * 95) / 100, // 5% slippage tolerance
        to: e.current_contract_address(),
    }
}

fn create_sell_swap(e: &Env, token_in: Address, amount_to_sell: u128) -> SwapParams {
    let base_token = get_base_token(e);

    SwapParams {
        provider: None, // Use default provider
        token_in,
        token_out: base_token,
        amount_in: amount_to_sell as i128,
        amount_out_min: ((amount_to_sell as i128) * 95) / 100, // 5% slippage tolerance
        to: e.current_contract_address(),
    }
}

fn get_base_token(e: &Env) -> Address {
    // Returns the index token as base
    get_token_share(e)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexDistribution {
    pub protocol_id: String,
    pub path: Vec<Address>,
    pub parts: u32,
}

pub fn get_default_distribution(e: &Env) -> Vec<DexDistribution> {
    let mut distribution = Vec::new(e);
    distribution.push_back(DexDistribution {
        protocol_id: String::from_str(e, "soroswap"),
        path: Vec::from_array(&e, [Address::from_str(e, "direct")]),
        parts: 100,
    });
    distribution
}
