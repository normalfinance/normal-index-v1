use adapter::{AdapterError, AdapterTrait};
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, Address, Env, String, Vec,
};
use types::adapter::AdapterTradeParams;
use utils::math::safe_math::SafeConversion;

contractmeta!(
    key = "Description",
    val = "An adapter for swapping tokens using Aquarius AMM pools"
);

#[contract]
pub struct AquariusAdapter;

#[contractimpl]
impl AquariusAdapter {
    /// Initializes adapter configuration for Aquarius pools.
    pub fn __constructor(e: Env, admin: Address, protocol_id: String, protocol_address: Address) {
        admin.require_auth();

        if crate::storage::get_initialized(&e) == true {
            panic_with_error!(&e, AdapterError::NotInitialized);
        }

        crate::storage::set_initialized(&e, &true);
        crate::storage::set_admin(&e, &admin);
        crate::storage::set_protocol_id(&e, &protocol_id);
        crate::storage::set_protocol_address(&e, &protocol_address);
    }
}

#[contractimpl]
impl AdapterTrait for AquariusAdapter {
    /// Executes a swap through Aquarius using exact-input routing.
    fn swap(e: Env, params: AdapterTradeParams) -> Result<u128, AdapterError> {
        params.to.require_auth();

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

    /// Returns the configured upstream protocol identifier.
    fn get_protocol_id(e: &Env) -> Result<String, AdapterError> {
        Ok(crate::storage::get_protocol_id(&e))
    }

    /// Returns the configured upstream protocol address.
    fn get_protocol_address(e: &Env) -> Result<Address, AdapterError> {
        Ok(crate::storage::get_protocol_address(&e))
    }
}
