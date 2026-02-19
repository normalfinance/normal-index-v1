use adapter::{AdapterError, AdapterTrait};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use types::adapter::AdapterTradeParams;
use utils::math::safe_math::SafeConversion;

use crate::aquarius_router::AquariusRouterClient;

#[contract]
pub struct AquariusAdapter;

#[contractimpl]
impl AquariusAdapter {
    pub fn __constructor(e: Env, admin: Address, protocol_id: String, protocol_address: Address) {
        crate::storage::set_admin(&e, &admin);
        crate::storage::set_protocol_id(&e, &protocol_id);
        crate::storage::set_protocol_address(&e, &protocol_address);
    }
}

#[contractimpl]
impl AdapterTrait for AquariusAdapter {
    fn swap(e: Env, params: AdapterTradeParams) -> Result<u128, AdapterError> {
        params.to.require_auth();

        // Set up Aquarius router client
        let aquarius_router_address = crate::storage::get_protocol_address(&e);
        let aquarius_router_client = AquariusRouterClient::new(&e, &aquarius_router_address);

        let path = Vec::from_array(&e, [params.token_in, params.token_out]);

        let amount_out = crate::aquarius_router::protocol_swap_exact_tokens_for_tokens(
            &e,
            &params.amount_in.safe_to_i128(&e),
            &params.amount_out_min.safe_to_i128(&e),
            &path,
            &params.to,
            &None,
        )?;

        Ok(amount_out)
    }

    fn get_protocol_id(e: &Env) -> Result<String, AdapterError> {
        Ok(crate::storage::get_protocol_id(&e))
    }

    fn get_protocol_address(e: &Env) -> Result<Address, AdapterError> {
        Ok(crate::storage::get_protocol_address(&e))
    }
}
