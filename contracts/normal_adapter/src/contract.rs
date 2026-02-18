use adapter::AdapterTrait;
use soroban_sdk::{contract, contractimpl, Address, Env, IntoVal, Symbol, Vec};
use types::adapter::AdapterTradeParams;

#[contract]
pub struct NormalAdapter;

#[contractimpl]
impl NormalAdapter {
    pub fn __constructor(e: Env, admin: Address, treasury: Address) {
        crate::storage::set_admin(&e, &admin);
        crate::storage::set_treasury(&e, &treasury);
    }
}

#[contractimpl]
impl AdapterTrait for NormalAdapter {
    fn buy(e: Env, params: AdapterTradeParams) -> u128 {
        let result = e.try_invoke_contract::<(u128, u128), soroban_sdk::Error>(
            &crate::storage::get_treasury(&e),
            &Symbol::new(&e, "buy_long"),
            Vec::from_array(&e, [params.into_val(&e)]),
        );

        match result {
            Ok(Ok(swap_result)) => swap_result.0,
            Err(_) => 0,
            Ok(Err(_)) => 0,
        }
    }

    fn sell(e: Env, params: AdapterTradeParams) -> u128 {
        let result = e.try_invoke_contract::<(u128, u128), soroban_sdk::Error>(
            &crate::storage::get_treasury(&e),
            &Symbol::new(&e, "sell_long"),
            Vec::from_array(&e, [params.into_val(&e)]),
        );

        match result {
            Ok(Ok(swap_result)) => swap_result.0,
            Err(_) => 0,
            Ok(Err(_)) => 0,
        }
    }
}
