use adapter::{AdapterError, AdapterTrait};
use core::convert::TryFrom;
use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, String, Symbol, Vec};
use types::adapter::AdapterTradeParams;
use utils::math::safe_math::SafeConversion;

use crate::soroswap_router::SorowswapRouterClient;

contractmeta!(
    key = "Description",
    val = "An adapter for swapping tokens using the Soroswap AMM Aggregator"
);

#[contract]
pub struct SoroswapAdapter;

#[contractimpl]
impl SoroswapAdapter {
    /// Initializes adapter configuration for the Soroswap router.
    pub fn __constructor(e: Env, admin: Address, protocol_id: String, protocol_address: Address) {
        crate::storage::set_admin(&e, &admin);
        crate::storage::set_protocol_id(&e, &protocol_id);
        crate::storage::set_protocol_address(&e, &protocol_address);
    }
}

#[contractimpl]
impl AdapterTrait for SoroswapAdapter {
    /// Executes a swap through Soroswap using exact-input routing.
    fn swap(e: Env, params: AdapterTradeParams) -> Result<u128, AdapterError> {
        params.to.require_auth();

        // Set up Soroswap router client
        let soroswap_router_address = crate::storage::get_protocol_address(&e);
        let soroswap_router_client = SorowswapRouterClient::new(&e, &soroswap_router_address);

        let path = Vec::from_array(&e, [params.token_in, params.token_out]);
        let deadline = match params
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.number.as_ref())
            .and_then(|numbers| numbers.get(Symbol::new(&e, "deadline")))
        {
            Some(value) => u64::try_from(value).map_err(|_| AdapterError::InvalidArgument)?,
            None => u64::MAX,
        };

        let result = soroswap_router_client.swap_exact_tokens_for_tokens(
            &params.amount_in.safe_to_i128(&e),
            &params.amount_out_min.safe_to_i128(&e),
            &path,
            &params.to,
            &deadline,
        );

        let total_swapped_amount = result.last().unwrap();

        Ok(total_swapped_amount.safe_to_u128(&e))
    }

    /// Returns the configured upstream protocol identifier.
    fn get_protocol_id(e: &Env) -> Result<String, AdapterError> {
        Ok(crate::storage::get_protocol_id(&e))
    }

    /// Returns the configured upstream protocol address.
    fn get_protocol_address(e: &Env) -> Result<Address, AdapterError> {
        Ok(crate::storage::get_protocol_address(&e))
    }
}
