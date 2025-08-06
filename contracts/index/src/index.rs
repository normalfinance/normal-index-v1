use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, IntoVal, String, Symbol, Vec};
use utils::{constant::FIVE_MINUTE, math::safe_math::SafeMath, validate};

use crate::errors::IndexError;
use crate::events::{Events, IndexEvents};
use crate::storage::{
    get_all_components, get_component_balance, get_factory, get_index_vault_amount,
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

        // Simplified swap execution - placeholder implementation
        let swap_result: Vec<Vec<i128>> = e.invoke_contract(
            &get_factory(&e),
            &Symbol::new(&e, "swap"),
            Vec::from_array(
                &e,
                [
                    e.current_contract_address().into_val(e),
                    params.token_in.into_val(e),
                    params.token_out.into_val(e),
                ],
            ),
        );

        // Placeholder event emission
        Events::new(&e).swap(
            Vec::new(&e),
            e.current_contract_address(),
            Symbol::new(&e, "pool"),
            params.token_in,
            params.token_out,
            params.amount_in,
            0i128,
        );

        // Add result to results vector
        results.push_back(0u128); // Placeholder result
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
