use soroban_sdk::{contracttype, Address, Map, Symbol};
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterMetadata {
    pub address: Option<Map<Symbol, Address>>,
    pub number: Option<Map<Symbol, i128>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterTradeParams {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: u128,
    pub amount_out_min: u128,
    pub to: Address,
    pub asset: Symbol,
    pub metadata: Option<AdapterMetadata>,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterResult {
    pub amount_in: u128,
    pub amount_out: u128,
    pub success: bool,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdapterError {
    ProviderNotSupported = 100,
    ProviderNotConfigured = 101,
    InvalidTokenPair = 200,
    InvalidAmount = 201,
    InvalidSlippage = 202,
    InsufficientLiquidity = 300,
    SlippageExceeded = 301,
    SwapFailed = 302,
    NormalDexFailed = 400,
    SoroswapSwapFailed = 401,
    SoroswapAggregatorUnavailable = 402,
    InvalidProviderConfig = 500,
    UnauthorizedAccess = 501,
    ContractNotInitialized = 502,
}

impl From<soroban_sdk::Error> for AdapterError {
    fn from(_: soroban_sdk::Error) -> Self {
        AdapterError::SwapFailed
    }
}
