use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SwapError {
    // Provider Errors
    ProviderNotSupported = 100,
    ProviderNotConfigured = 101,

    // Parameter Validation Errors
    InvalidTokenPair = 200,
    InvalidAmount = 201,
    InvalidSlippage = 202,

    // Execution Errors
    InsufficientLiquidity = 300,
    SlippageExceeded = 301,
    SwapFailed = 302,

    // Provider-Specific Errors
    NormalDexFailed = 400,
    SoroswapSwapFailed = 401,
    SoroswapAggregatorUnavailable = 402,

    // Configuration Errors
    InvalidProviderConfig = 500,
    UnauthorizedAccess = 501,
    ContractNotInitialized = 502,
}
