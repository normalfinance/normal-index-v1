use index_access_control::errors::IndexAccessControlError;
use soroban_sdk::{panic_with_error, Address, Env, Symbol, Vec};
use types::oracle::{HistoricalOracleData, OracleValidity};

/// Oracle helper methods used by index valuation and conversions.
pub struct OracleUtils;

impl OracleUtils {
    /// Gets the oracle address for a token from its Component storage
    ///
    /// # Arguments
    /// - `env` (`&Env`): Soroban environment.
    /// - `token` (`&Address`): Component token address.
    ///
    /// # Returns
    /// - `Address`: Oracle contract address configured for the component.
    fn get_component_oracle(env: &Env, token: &Address) -> Address {
        let component = crate::storage::get_component(env, token.clone());
        component.oracle
    }

    /// Get token price from oracle, returns None if oracle call fails
    ///
    /// # Arguments
    /// - `env` (`&Env`): Soroban environment.
    /// - `oracle` (`&Address`): Oracle contract address.
    ///
    /// # Returns
    /// - `Option<u128>`: Price in oracle units, or `None` when no valid price is available.
    pub fn get_token_price_usd_safe(env: &Env, oracle: &Address) -> Option<u128> {
        let result = env
            .try_invoke_contract::<(HistoricalOracleData, OracleValidity), IndexAccessControlError>(
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

    /// Reads a token price from an oracle contract and panics when no usable value exists.
    ///
    /// # Arguments
    /// - `env` (`&Env`): Soroban environment.
    /// - `oracle` (`&Address`): Oracle contract address.
    ///
    /// # Returns
    /// - `u128`: Valid token price returned by the oracle.
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
                    panic_with_error!(env, IndexAccessControlError::Unauthorized);
                }
                OracleValidity::TooVolatile => historical_data.last_price_twap,
                OracleValidity::StaleForPool | OracleValidity::Frozen => {
                    panic_with_error!(env, IndexAccessControlError::Unauthorized);
                }
            },
            Err(_) => {
                panic_with_error!(env, IndexAccessControlError::Unauthorized);
            }
        }
    }

    /// Converts a token amount to USD units using the configured component oracle.
    ///
    /// # Arguments
    /// - `env` (`&Env`): Soroban environment.
    /// - `token` (`&Address`): Component token address.
    /// - `amount` (`u128`): Token amount in token base units.
    ///
    /// # Returns
    /// - `u128`: USD value scaled by the oracle decimal convention.
    pub fn convert_token_to_usd(env: &Env, token: &Address, amount: u128) -> u128 {
        let oracle = Self::get_component_oracle(env, token);
        let token_price_usd = Self::get_token_price_usd(env, &oracle);

        amount
            .saturating_mul(token_price_usd)
            .saturating_div(10_000_000)
    }

    /// Safe version of convert_token_to_usd that returns None if oracle fails
    ///
    /// # Arguments
    /// - `env` (`&Env`): Soroban environment.
    /// - `token` (`&Address`): Component token address.
    /// - `amount` (`u128`): Token amount in token base units.
    ///
    /// # Returns
    /// - `Option<u128>`: Converted USD value, or `None` on oracle failure/invalidity.
    pub fn convert_token_to_usd_safe(env: &Env, token: &Address, amount: u128) -> Option<u128> {
        let oracle = Self::get_component_oracle(env, token);

        Self::get_token_price_usd_safe(env, &oracle).map(|token_price_usd| {
            amount
                .saturating_mul(token_price_usd)
                .saturating_div(10_000_000)
        })
    }
}
