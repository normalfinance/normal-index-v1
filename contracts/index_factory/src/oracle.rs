use crate::storage::get_oracle_registry;
use access_control::errors::AccessControlError;
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{
    contractclient, contracttype, panic_with_error, Address, Env, IntoVal, String, Symbol, Vec,
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub struct HistoricalOracleData {
    pub last_oracle_price: u128,
    pub last_oracle_price_twap: u128,
    pub last_oracle_price_twap_ts: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub enum OracleValidity {
    NonPositive,
    TooVolatile,
    StaleForPool,
    Frozen,
    Valid,
}

impl Default for OracleValidity {
    fn default() -> Self {
        Self::Valid
    }
}

#[contractclient(name = "OracleRegistryClient")]
pub trait OracleRegistry {
    fn get_price(env: Env, asset: Symbol) -> (HistoricalOracleData, OracleValidity);
}

pub struct OracleUtils;

impl OracleUtils {
    pub fn convert_token_to_usd(env: &Env, token: &Address, amount: u128) -> u128 {
        let oracle_registry = get_oracle_registry(env);

        let token_client = SorobanTokenClient::new(env, token);
        let token_symbol = token_client.symbol();
        let stub_symbol = Symbol::new(env, "XLM");
        // FIXME: convert String to Symbol
        // let symbol = Symbol::new(env, &token_symbol);

        let token_price_usd = Self::get_token_price_usd(env, &stub_symbol, &oracle_registry);

        amount
            .saturating_mul(token_price_usd)
            .saturating_div(10_000_000)
    }

    pub fn convert_xlm_to_usd(env: &Env, xlm_amount: u128) -> u128 {
        let oracle_registry = get_oracle_registry(env);
        let xlm_price_usd =
            Self::get_token_price_usd(env, &Symbol::new(env, "XLM"), &oracle_registry);

        xlm_amount
            .saturating_mul(xlm_price_usd)
            .saturating_div(10_000_000)
    }

    pub fn get_token_price_usd(env: &Env, asset: &Symbol, oracle_registry: &Address) -> u128 {
        let result: Result<(HistoricalOracleData, OracleValidity), soroban_sdk::Error> = env
            .invoke_contract(
                oracle_registry,
                &Symbol::new(env, "get_price"),
                Vec::from_array(env, [asset.clone().into_val(env)]),
            );

        match result {
            Ok((historical_data, validity)) => match validity {
                OracleValidity::Valid => historical_data.last_oracle_price,
                OracleValidity::NonPositive => {
                    panic_with_error!(env, AccessControlError::Unauthorized);
                }
                OracleValidity::TooVolatile => historical_data.last_oracle_price_twap,
                OracleValidity::StaleForPool | OracleValidity::Frozen => {
                    panic_with_error!(env, AccessControlError::Unauthorized);
                }
            },
            Err(_) => {
                panic_with_error!(env, AccessControlError::Unauthorized);
            }
        }
    }
}
