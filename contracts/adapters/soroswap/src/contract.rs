use adapter::{AdapterError, AdapterTrait};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use types::adapter::AdapterTradeParams;
use utils::math::safe_math::SafeConversion;

use crate::soroswap_router::SorowswapRouterClient;

#[contract]
pub struct SoroswapAdapter;

#[contractimpl]
impl SoroswapAdapter {
    pub fn __constructor(e: Env, admin: Address, protocol_id: String, protocol_address: Address) {
        crate::storage::set_admin(&e, &admin);
        crate::storage::set_protocol_id(&e, &protocol_id);
        crate::storage::set_protocol_address(&e, &protocol_address);
    }
}

#[contractimpl]
impl AdapterTrait for SoroswapAdapter {
    fn swap(e: Env, params: AdapterTradeParams) -> Result<u128, AdapterError> {
        params.to.require_auth();

        // Set up Soroswap router client
        let soroswap_router_address = crate::storage::get_protocol_address(&e);
        let soroswap_router_client = SorowswapRouterClient::new(&e, &soroswap_router_address);

        let path = Vec::from_array(&e, [params.token_in, params.token_out]);

        let result = soroswap_router_client.swap_exact_tokens_for_tokens(
            &params.amount_in.safe_to_i128(&e),
            &params.amount_out_min.safe_to_i128(&e),
            &path,
            &params.to,
            &u64::MAX,
        );

        let total_swapped_amount = result.last().unwrap();

        Ok(total_swapped_amount.safe_to_u128(&e))
    }

    fn get_protocol_id(e: &Env) -> Result<String, AdapterError> {
        Ok(crate::storage::get_protocol_id(&e))
    }

    fn get_protocol_address(e: &Env) -> Result<Address, AdapterError> {
        Ok(crate::storage::get_protocol_address(&e))
    }
}
