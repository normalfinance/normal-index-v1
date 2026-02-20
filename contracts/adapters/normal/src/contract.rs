use adapter::{AdapterError, AdapterTrait};
use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, String, Symbol};
use types::adapter::AdapterTradeParams;

use crate::normal_treasury::{Direction, NormalTreasuryClient, Side};

contractmeta!(
    key = "Description",
    val = "An adapter for buying and selling long and short Normal tokens via the Normal Treasury"
);

#[contract]
pub struct NormalAdapter;

#[contractimpl]
impl NormalAdapter {
    /// Initializes adapter configuration for the Normal treasury protocol.
    pub fn __constructor(
        e: Env,
        admin: Address,
        protocol_id: String,
        protocol_address: Address,
        protocol_quote_token: Address,
    ) {
        crate::storage::set_admin(&e, &admin);
        crate::storage::set_protocol_id(&e, &protocol_id);
        crate::storage::set_protocol_address(&e, &protocol_address);
        crate::storage::set_protocol_quote_token(&e, &protocol_quote_token);
    }
}

#[contractimpl]
impl AdapterTrait for NormalAdapter {
    /// Executes a swap through the Normal treasury using pair metadata.
    fn swap(e: Env, params: AdapterTradeParams) -> Result<u128, AdapterError> {
        let normal_treasury_address = crate::storage::get_protocol_address(&e);
        let normal_treasury_client = NormalTreasuryClient::new(&e, &normal_treasury_address);

        let protocol_quote_token = crate::storage::get_protocol_quote_token(&e);

        let direction = if params.token_in == protocol_quote_token {
            Direction::Buy
        } else {
            Direction::Sell
        };

        let side = Side::Long;

        let mut amount_out = 0;

        let pair = params
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.address.as_ref())
            .and_then(|addresses| addresses.get(Symbol::new(&e, "pair")))
            .ok_or(AdapterError::InvalidArgument)?;

        match (direction, side) {
            (Direction::Buy, Side::Long) => {
                let (long_out, _) = normal_treasury_client.buy_long(
                    &params.to,
                    &pair,
                    &params.amount_in,
                    &params.amount_out_min,
                );

                amount_out = long_out;
            }
            (Direction::Buy, Side::Short) => {
                let (short_out, _) = normal_treasury_client.buy_short(
                    &params.to,
                    &pair,
                    &params.amount_in,
                    &params.amount_out_min,
                );

                amount_out = short_out;
            }
            (Direction::Sell, Side::Long) => {
                let (usdc_out, _) = normal_treasury_client.sell_long(
                    &params.to,
                    &pair,
                    &params.amount_in,
                    &params.amount_out_min,
                );

                amount_out = usdc_out;
            }
            (Direction::Sell, Side::Short) => {
                let (usdc_out, _) = normal_treasury_client.sell_short(
                    &params.to,
                    &pair,
                    &params.amount_in,
                    &params.amount_out_min,
                );

                amount_out = usdc_out;
            }
        }

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
