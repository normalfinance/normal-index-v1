use soroban_sdk::{ Env, Vec };
use utils::{ constant::FIVE_MINUTE, math::safe_math::SafeMath, validate };

use crate::storage::{ get_all_components, get_component_balance, get_index_vault_amount };

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

    let distribution = Vec::new(e).push_back(DexDistribution {
        protocol_id: "",
        path: "",
        parts: "",
    });
    let components = get_all_components(e);

    let swaps: Vec<SwapParams>;

    for i in 0..components.len() {
        let current_balance = get_component_balance(e, token) as i128;
        let target_balance = 0_i128;

        let delta = target_balance.safe_sub(e, current_balance);

        let swap = SwapParams {
            token_in: if delta > 0 {
                component.asset
            } else {
                XLM
            },
            token_out: if delta > 0 {
                XLM
            } else {
                component.asset
            },
            amount_in: delta,
            amount_out_min: 0,
            distribution,
            to: e.current_contract_address(),
            deadline: now + FIVE_MINUTE,
        };

        swaps.push_back(swap);
    }

    swaps
}

pub fn execute_swaps(e: &Env, swaps: Vec<SwapParams>) -> Vec<u128> {
    for i in 0..swaps.len() {
        let params = swaps.get(i).unwrap();

        e.authorize_as_current_contract(
            vec![
                &e,
                InvokerContractAuthEntry::Contract(SubContractInvocation {
                    context: ContractContext {
                        contract: token_in_local.clone(),
                        fn_name: Symbol::new(&e, "transfer"),
                        args: (
                            e.current_contract_address(),
                            pool_id.clone(),
                            in_amount_local as i128,
                        ).into_val(&e),
                    },
                    sub_invocations: vec![&e],
                })
            ]
        );

        let swap_result: Vec<Vec<i128>> = e.invoke_contract(
            &get_factory(&e),
            &symbol_short!("swap"),
            Vec::from_array(&e, [
                e.current_contract_address().into_val(&e),
                params.token_in,
                params.token_out,
            ])
        );

        Events::new(&e).swap(
            tokens,
            user.clone(),
            pool_id,
            token_in_local.clone(),
            token_out.clone(),
            in_amount_local,
            last_swap_result
        );
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
        amount.fixed_mul_floor(e, &total_shares, &vault_amount)
        // get_proportion_u128(e, amount, total_shares, vault_amount)
    } else {
        // must be case that total_shares == 0 for nice result for user
        validate!(
            e,
            total_shares == 0,
            InsuranceFundError::InvalidIFSharesDetected,
            "assumes total_shares == 0"
        );

        amount
    };

    n_shares
}
