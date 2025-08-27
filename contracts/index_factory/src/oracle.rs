use crate::storage::get_oracle_registry;
use access_control::errors::AccessControlError;
use soroban_sdk::{
    contractclient, contracttype, panic_with_error, Address, Env, IntoVal, Symbol, Vec,
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
    fn get_price(env: Env, asset: Address) -> (HistoricalOracleData, OracleValidity);
}

pub struct OracleUtils;

impl OracleUtils {
    pub fn convert_xlm_to_usd(env: &Env, xlm_asset: &Address, xlm_amount: u128) -> u128 {
        let oracle_registry = get_oracle_registry(env);
        let xlm_price_usd = Self::get_xlm_price_usd(env, &oracle_registry, xlm_asset);

        xlm_amount
            .saturating_mul(xlm_price_usd)
            .saturating_div(10_000_000)
    }

    pub fn get_xlm_price_usd(env: &Env, oracle_registry: &Address, xlm_asset: &Address) -> u128 {
        let result: Result<(HistoricalOracleData, OracleValidity), soroban_sdk::Error> = env
            .invoke_contract(
                oracle_registry,
                &Symbol::new(env, "get_price"),
                Vec::from_array(env, [xlm_asset.clone().into_val(env)]),
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
