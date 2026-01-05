use access_control::errors::AccessControlError;
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{
    contractclient, contracttype, panic_with_error, Address, Env, IntoVal, String, Symbol, Vec,
};
use types::oracle::{HistoricalOracleData, OraclePriceData, OracleValidity};

#[contractclient(name = "NormalOracleClient")]
pub trait NormalOracle {
    fn get_price(e: Env) -> OraclePriceData;
}

pub struct OracleUtils;

impl OracleUtils {
    /// Converts a Soroban String to a Symbol for oracle lookup
    fn string_to_symbol(env: &Env, s: &String) -> Symbol {
        let len = s.len() as usize;
        if len == 0 || len > 32 {
            return Symbol::new(env, "UNKNOWN");
        }

        let mut bytes = [0u8; 32];
        s.copy_into_slice(&mut bytes[..len]);

        // Convert bytes to str (safe because token symbols should be ASCII)
        match core::str::from_utf8(&bytes[..len]) {
            Ok(str_slice) => Symbol::new(env, str_slice),
            Err(_) => Symbol::new(env, "UNKNOWN"),
        }
    }

    pub fn convert_token_to_usd(env: &Env, token: &Address, amount: u128) -> u128 {
        let token_client = SorobanTokenClient::new(env, token);
        let token_symbol_string = token_client.symbol();

        // Convert Soroban String to Symbol for oracle lookup
        let symbol = Self::string_to_symbol(env, &token_symbol_string);

        let token_price_usd = Self::get_token_price_usd(env, &symbol, &oracle);

        amount
            .saturating_mul(token_price_usd)
            .saturating_div(10_000_000)
    }

    /// Get token price from oracle, returns None if oracle call fails
    pub fn get_token_price_usd_safe(env: &Env, oracle: &Address) -> Option<u128> {
        let result = env.try_invoke_contract::<OraclePriceData, AccessControlError>(
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
        let result: Result<OraclePriceData, soroban_sdk::Error> = env.invoke_contract(
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

    /// Safe version of convert_token_to_usd that returns None if oracle fails
    pub fn convert_token_to_usd_safe(env: &Env, token: &Address, amount: u128) -> Option<u128> {
        let token_client = SorobanTokenClient::new(env, token);
        let token_symbol_string = token_client.symbol();

        // Convert Soroban String to Symbol for oracle lookup
        let symbol = Self::string_to_symbol(env, &token_symbol_string);

        Self::get_token_price_usd_safe(env, &symbol, &oracle).map(|token_price_usd| {
            amount
                .saturating_mul(token_price_usd)
                .saturating_div(10_000_000)
        })
    }
}
