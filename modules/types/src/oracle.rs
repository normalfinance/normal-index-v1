use soroban_sdk::{contracterror, contracttype};

#[contracttype]
#[derive(Clone)]
pub enum OracleSource {
    Reflector,
}

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum OracleError {
    #[doc = "OracleError: OracleNonPositive"]
    OracleNonPositive = 601,
    OracleTooVolatile = 602,
    OracleStaleForPool = 603,
    OracleInvalid = 604,
    FailedToGetOraclePrice = 605,
}

#[contracttype]
#[derive(Default, Clone, Copy, Debug)]
pub struct OraclePriceData {
    pub price: u128,
    pub delay: u64,
}

// ordered by "severity"
#[contracttype]
#[derive(Clone, Copy, PartialEq, Debug, Eq, Default)]
pub enum OracleValidity {
    NonPositive,
    TooVolatile,
    StaleForPool,
    Frozen,
    #[default]
    Valid,
}

impl OracleValidity {
    pub fn get_error_code(&self) -> OracleError {
        match self {
            OracleValidity::NonPositive => OracleError::OracleNonPositive,
            OracleValidity::TooVolatile => OracleError::OracleTooVolatile,
            OracleValidity::StaleForPool => OracleError::OracleStaleForPool,
            OracleValidity::Frozen => unreachable!(),
            OracleValidity::Valid => unreachable!(),
        }
    }
}

#[contracttype]
#[derive(Default, Clone, Copy, Eq, PartialEq, Debug)]
pub struct HistoricalOracleData {
    pub last_price: u128,
    pub last_price_twap: u128,
    pub last_update_ts: u64, // unix_timestamp of last snapshot.
}

impl HistoricalOracleData {
    pub fn default_quote_oracle() -> Self {
        HistoricalOracleData {
            last_price: 10_000_000_u128,
            last_price_twap: 10_000_000_u128,
            ..HistoricalOracleData::default()
        }
    }

    pub fn default_price(price: u128) -> Self {
        HistoricalOracleData {
            last_price: price,
            last_price_twap: price,
            ..HistoricalOracleData::default()
        }
    }

    pub fn default_with_current_oracle(price_data: OraclePriceData) -> Self {
        HistoricalOracleData {
            last_price: price_data.price,
            last_price_twap: price_data.price,
            ..HistoricalOracleData::default()
        }
    }
}
