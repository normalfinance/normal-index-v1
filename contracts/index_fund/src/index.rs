use soroban_sdk::{panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use types::adapter::AdapterTradeParams;
use types::component::RebalanceParams;
use utils::validate;

use crate::errors::IndexFundError;
use crate::events::{Events, IndexEvents};
use crate::interface::QueryInterface;
use crate::storage::get_all_components;

pub fn generate_swap_params(
    e: &Env,
    deposit_token: Address,
    deposit_amount: u128,
) -> Vec<AdapterTradeParams> {
    // Validate inputs
    validate!(e, deposit_amount > 0, IndexFundError::PathIsEmpty);

    let components = get_all_components(e);
    let mut swaps: Vec<AdapterTradeParams> = Vec::new(e);

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
            let min_output = (target_allocation * 95) / 100;
            validate!(e, min_output > 0, IndexFundError::PathIsEmpty);

            //Revisit this param generation here
            let swap: AdapterTradeParams = AdapterTradeParams {
                token_in: deposit_token.clone(),
                token_out: component_address.clone(),
                amount_in: target_allocation,
                amount_out_min: min_output,
                to: e.current_contract_address(),
                asset: component.asset.clone(),
                metadata: None,
            };

            swaps.push_back(swap);
        }
    }

    swaps
}

pub fn execute_swaps(e: &Env, swaps: Vec<AdapterTradeParams>) -> Vec<u128> {
    let quote_token = crate::storage::get_token_quote(e);

    let mut results: Vec<u128> = Vec::new(e);

    for i in 0..swaps.len() {
        let params = swaps.get(i).unwrap();

        // Get the component info to extract the asset symbol
        // For buy swaps: token_out is the component, for sell swaps: token_in is the component
        let component_token = if params.token_out == quote_token {
            // Sell swap: selling component for base token
            params.token_in.clone()
        } else {
            // Buy swap: buying component with base token
            params.token_out.clone()
        };
        let component = crate::storage::get_component(e, component_token.clone());

        // let method = if params.token_out == quote_token {
        //     Symbol::new(e, "sell_token")
        // } else {
        //     Symbol::new(e, "buy_token")
        // };

        let adapter_address = crate::adapter::get_adapter_from_registry(e, &component.adapter);

        match adapter_result {
            Ok(Ok(amount_out)) => {
                Events::new(&e).swap(
                    Vec::new(&e),
                    e.current_contract_address(),
                    component.adapter,
                    params.token_in,
                    params.token_out,
                    params.amount_in,
                    amount_out,
                );
                results.push_back(amount_out);
            }
            Ok(Err(_swap_error)) => {
                Events::new(&e).swap_failed(
                    e.current_contract_address(),
                    params.token_in,
                    params.token_out,
                    params.amount_in,
                    1000u32,
                );
                results.push_back(0u128);
            }
            Err(_contract_error) => {
                Events::new(&e).swap_failed(
                    e.current_contract_address(),
                    params.token_in,
                    params.token_out,
                    params.amount_in,
                    999u32,
                );
                results.push_back(0u128);
            }
        }
    }

    results
}

// Enhanced rebalancing swap generation - now focuses on balancing existing components
pub fn generate_rebalance_swaps(e: &Env, params: &RebalanceParams) -> Vec<AdapterTradeParams> {
    let current_nav = crate::IndexFund::get_current_nav(e.clone()); // Simplified NAV calculation
    let target_nav = params.target_nav.map(|n| n as u128).unwrap_or(current_nav);

    let quote_token = crate::storage::get_token_quote(e);

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

        let current_balance =
            crate::storage::get_component_balance_safe(e, token_address.clone()).unwrap_or(0);

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
                let swap = AdapterTradeParams {
                    token_in: quote_token.clone(),
                    token_out: token_address.clone(),
                    amount_in: amount_needed,
                    amount_out_min: (amount_needed * 95) / 100, // 5% slippage tolerance
                    to: e.current_contract_address(),
                    asset: component.asset.clone(),
                    metadata: None,
                };

                swaps.push_back(swap);
            }
        } else if target_balance < current_balance {
            // Need to sell some of this component
            let amount_to_sell = current_balance - target_balance;
            if amount_to_sell > 0 {
                let swap = AdapterTradeParams {
                    token_in: token_address.clone(),
                    token_out: quote_token.clone(),
                    amount_in: amount_to_sell,
                    amount_out_min: (amount_to_sell * 95) / 100, // 5% slippage tolerance
                    to: e.current_contract_address(),
                    asset: component.asset.clone(),
                    metadata: None,
                };

                swaps.push_back(swap);
            }
        }
        // If target_balance == current_balance, no swap needed for this component
    }

    swaps
}
