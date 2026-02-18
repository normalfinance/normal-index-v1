use access_control::errors::AccessControlError;
use soroban_sdk::{panic_with_error, Address, Env, Symbol, Vec};
use types::oracle::{HistoricalOracleData, OracleValidity};

pub struct OracleUtils;

impl OracleUtils {
    /// Gets the oracle address for a token from its Component storage
    fn get_component_oracle(env: &Env, token: &Address) -> Address {
        let component = crate::storage::get_component(env, token.clone());
        component.oracle
    }

    /// Get token price from oracle, returns None if oracle call fails
    pub fn get_token_price_usd_safe(env: &Env, oracle: &Address) -> Option<u128> {
        let result = env
            .try_invoke_contract::<(HistoricalOracleData, OracleValidity), AccessControlError>(
                oracle,
                &Symbol::new(env, "get_price"),
                Vec::from_array(env, []),
            );

        match result {
            Ok(Ok((historical_data, validity))) => match validity {
                OracleValidity::Valid => Some(historical_data.last_price),
                OracleValidity::TooVolatile => Some(historical_data.last_price_twap),
                OracleValidity::NonPositive
                | OracleValidity::StaleForPool
                | OracleValidity::Frozen => None,
            },
            _ => None,
        }
    }

    pub fn get_token_price_usd(env: &Env, oracle: &Address) -> u128 {
        let result: Result<(HistoricalOracleData, OracleValidity), soroban_sdk::Error> = env
            .invoke_contract(
                oracle,
                &Symbol::new(env, "get_price"),
                Vec::from_array(env, []),
            );

        match result {
            Ok((historical_data, validity)) => match validity {
                OracleValidity::Valid => historical_data.last_price,
                OracleValidity::NonPositive => {
                    panic_with_error!(env, AccessControlError::Unauthorized);
                }
                OracleValidity::TooVolatile => historical_data.last_price_twap,
                OracleValidity::StaleForPool | OracleValidity::Frozen => {
                    panic_with_error!(env, AccessControlError::Unauthorized);
                }
            },
            Err(_) => {
                panic_with_error!(env, AccessControlError::Unauthorized);
            }
        }
    }

    pub fn convert_token_to_usd(env: &Env, token: &Address, amount: u128) -> u128 {
        let oracle = Self::get_component_oracle(env, token);
        let token_price_usd = Self::get_token_price_usd(env, &oracle);

        amount
            .saturating_mul(token_price_usd)
            .saturating_div(10_000_000)
    }

    /// Safe version of convert_token_to_usd that returns None if oracle fails
    pub fn convert_token_to_usd_safe(env: &Env, token: &Address, amount: u128) -> Option<u128> {
        let oracle = Self::get_component_oracle(env, token);

        Self::get_token_price_usd_safe(env, &oracle).map(|token_price_usd| {
            amount
                .saturating_mul(token_price_usd)
                .saturating_div(10_000_000)
        })
    }
}
