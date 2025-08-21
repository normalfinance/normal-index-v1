use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, IntoVal, String, Symbol, Vec};
use utils::{constant::FIVE_MINUTE, math::safe_math::SafeMath, validate};

// Types to match the SwapUtility contract
#[derive(Clone)]
#[contracttype]
pub struct SwapUtilityParams {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
    pub amount_out_min: i128,
    pub to: Address,
    pub deadline: u64,
    pub provider: Option<String>, // DexProvider as string
}

#[derive(Clone)]
#[contracttype]
pub struct SwapResult {
    pub amount_in: i128,
    pub amount_out: i128,
    pub provider_used: String,
}

use crate::errors::IndexError;
use crate::events::{Events, IndexEvents};
use crate::storage::{
    get_all_components, get_component_balance, get_factory, get_index_vault_amount, get_swap_utility,
};

#[derive(Clone)]
#[contracttype]
pub struct DexDistribution {
    pub protocol_id: String,
    pub path: String,
    pub parts: String,
}

#[contracttype]
pub struct SwapParams {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
    pub amount_out_min: i128,
    pub distribution: Vec<DexDistribution>,
    pub to: Address,
    pub deadline: u64,
}

pub fn generate_swap_params(e: &Env, now: u64) -> Vec<SwapParams> {
    // diff b/t target state and current state
    // let baa

    let mut distribution: Vec<DexDistribution> = Vec::new(e);
    distribution.push_back(DexDistribution {
        protocol_id: String::from_str(&e, ""),
        path: String::from_str(&e, ""),
        parts: String::from_str(&e, ""),
    });
    let components = get_all_components(e);

    let mut swaps: Vec<SwapParams> = Vec::new(e);

    // Placeholder implementation - skip component iteration for now
    let current_balance = 0i128;
    let target_balance = 0_i128;

    let delta = target_balance.safe_sub(e, current_balance);

    let swap = SwapParams {
        token_in: if delta > 0 {
            Address::from_str(&e, "token1")
        } else {
            Address::from_str(&e, "token2")
        },
        token_out: if delta > 0 {
            Address::from_str(&e, "token2")
        } else {
            Address::from_str(&e, "token1")
        },
        amount_in: delta,
        amount_out_min: 0,
        distribution,
        to: e.current_contract_address(),
        deadline: now + FIVE_MINUTE as u64,
    };

    swaps.push_back(swap);

    swaps
}

pub fn execute_swaps(e: &Env, swaps: Vec<SwapParams>) -> Vec<u128> {
    let mut results: Vec<u128> = Vec::new(e);

    for i in 0..swaps.len() {
        let params = swaps.get(i).unwrap();

        // Map local SwapParams to SwapUtilityParams for the external contract
        let utility_params = SwapUtilityParams {
            token_in: params.token_in.clone(),
            token_out: params.token_out.clone(),
            amount_in: params.amount_in,
            amount_out_min: params.amount_out_min,
            to: params.to.clone(),
            deadline: params.deadline,
            provider: None, // Use default provider from SwapUtility
        };

        // Execute individual swap via cross-contract call to swap utility
        let swap_result: Result<Result<SwapResult, u32>, u32> = e.try_invoke_contract(
            &get_swap_utility(&e),
            &Symbol::new(&e, "execute_swap"),
            Vec::from_array(
                &e,
                [utility_params.into_val(e)],
            ),
        );

        match swap_result {
            Ok(Ok(swap_result)) => {
                // Successful swap - SwapResult from utility contract
                Events::new(&e).swap(
                    Vec::new(&e),
                    e.current_contract_address(),
                    Symbol::from_str(&e, &swap_result.provider_used),
                    params.token_in,
                    params.token_out,
                    swap_result.amount_in,
                    swap_result.amount_out,
                );

                // Add successful result
                results.push_back(swap_result.amount_out as u128);
            }
            Ok(Err(swap_error)) => {
                // Swap failed but call succeeded - emit failure event
                Events::new(&e).swap_failed(
                    e.current_contract_address(),
                    params.token_in,
                    params.token_out,
                    params.amount_in,
                    swap_error,
                );

                // Add zero result for failed swap
                results.push_back(0u128);
            }
            Err(contract_error) => {
                // Contract call failed - emit failure event with generic error
                Events::new(&e).swap_failed(
                    e.current_contract_address(),
                    params.token_in,
                    params.token_out,
                    params.amount_in,
                    contract_error,
                );

                // Add zero result for failed swap
                results.push_back(0u128);
            }
        }
    }

    results
}

pub fn vault_amount_to_shares(
    e: &Env,
    amount: u128,
    total_shares: u128,
    vault_amount: u128,
) -> u128 {
    // relative to the entire pool + total amount minted
    let n_shares = if vault_amount > 0 {
        // assumes total_shares != 0 (in most cases) for nice result for user
        amount
            .fixed_mul_floor(total_shares, vault_amount)
            .unwrap_or(amount)
        // get_proportion_u128(e, amount, total_shares, vault_amount)
    } else {
        // must be case that total_shares == 0 for nice result for user
        validate!(e, total_shares == 0, IndexError::InvalidIFSharesDetected);

        amount
    };

    n_shares
}

// Enhanced rebalancing swap generation
pub fn generate_rebalance_swaps(
    e: &Env,
    params: &crate::interface::RebalanceParams,
) -> Vec<SwapParams> {
    let current_nav = crate::storage::get_base_nav(e) as u128; // Simplified NAV calculation
    let target_nav = params.target_nav.map(|n| n as u128).unwrap_or(current_nav);

    let mut swaps = Vec::new(e);

    for update in params.component_updates.iter() {
        let current_balance =
            crate::storage::get_component_balance_safe(e, update.token.clone()).unwrap_or(0);

        match update.action {
            crate::interface::ComponentAction::Add => {
                // Calculate target balance for new component
                let target_balance = if target_nav > 0 {
                    (target_nav * update.new_weight as u128) / 10000
                } else {
                    0
                };

                if target_balance > current_balance {
                    // Need to buy this component
                    let swap =
                        create_buy_swap(e, update.token.clone(), target_balance - current_balance);
                    swaps.push_back(swap);
                }
            }
            crate::interface::ComponentAction::UpdateWeight => {
                // Calculate new target balance based on updated weight
                let target_balance = if target_nav > 0 {
                    (target_nav * update.new_weight as u128) / 10000
                } else {
                    0
                };

                if target_balance > current_balance {
                    // Need to buy more
                    let swap =
                        create_buy_swap(e, update.token.clone(), target_balance - current_balance);
                    swaps.push_back(swap);
                } else if target_balance < current_balance {
                    // Need to sell some
                    let swap =
                        create_sell_swap(e, update.token.clone(), current_balance - target_balance);
                    swaps.push_back(swap);
                }
            }
            crate::interface::ComponentAction::Remove => {
                // Sell all of this component
                if current_balance > 0 {
                    let swap = create_sell_swap(e, update.token.clone(), current_balance);
                    swaps.push_back(swap);
                }
            }
        }
    }

    swaps
}

fn create_buy_swap(e: &Env, token_out: Address, amount_needed: u128) -> SwapParams {
    let base_token = get_base_token(e);

    SwapParams {
        token_in: base_token,
        token_out,
        amount_in: amount_needed as i128, // Simplified 1:1 ratio
        amount_out_min: (amount_needed as i128 * 95) / 100, // 5% slippage tolerance
        distribution: get_default_distribution(e),
        to: e.current_contract_address(),
        deadline: e.ledger().timestamp() + utils::constant::FIVE_MINUTE as u64,
    }
}

fn create_sell_swap(e: &Env, token_in: Address, amount_to_sell: u128) -> SwapParams {
    let base_token = get_base_token(e);

    SwapParams {
        token_in,
        token_out: base_token,
        amount_in: amount_to_sell as i128,
        amount_out_min: (amount_to_sell as i128 * 95) / 100, // 5% slippage tolerance
        distribution: get_default_distribution(e),
        to: e.current_contract_address(),
        deadline: e.ledger().timestamp() + utils::constant::FIVE_MINUTE as u64,
    }
}

fn get_base_token(e: &Env) -> Address {
    // Returns the index token as base
    crate::storage::get_token(e)
}

fn get_default_distribution(e: &Env) -> Vec<DexDistribution> {
    let mut distribution = Vec::new(e);
    distribution.push_back(DexDistribution {
        protocol_id: String::from_str(e, "soroswap"),
        path: String::from_str(e, "direct"),
        parts: String::from_str(e, "100"),
    });
    distribution
}
