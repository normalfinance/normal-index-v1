#![no_std]

use soroban_sdk::{self, contracterror, Address, Env, String};
use types::adapter::AdapterTradeParams;

pub trait AdapterTrait {
    fn swap(e: Env, params: AdapterTradeParams) -> Result<u128, AdapterError>;

    fn get_protocol_id(e: &Env) -> Result<String, AdapterError>;
    fn get_protocol_address(e: &Env) -> Result<Address, AdapterError>;
}

pub trait AdapterAdminInterface {
    fn get_total_index_count(e: Env) -> u32;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdapterError {
    // Initialization Errors
    NotInitialized = 401,

    // Aquarius Errors
    MissingPoolHashes = 100,
    WrongMinimumPathLength = 101,
    WrongPoolHashesLength = 102,

    // Validation Errors
    NegativeNotAllowed = 410,
    InvalidArgument = 411,
    InsufficientBalance = 412,
    UnderflowOverflow = 413,
    ArithmeticError = 414,
    DivisionByZero = 415,
    InvalidSharesMinted = 416,
    OnlyPositiveAmountAllowed = 417,
    NotAuthorized = 418,

    // Protocol Errors
    ProtocolAddressNotFound = 420,
    DeadlineExpired = 421,
    ExternalError = 422,
    SoroswapPairError = 423,

    // Blend Errors
    AmountBelowMinDust = 451,
    UnderlyingAmountBelowMin = 452,
    BTokensAmountBelowMin = 453,
    InternalSwapError = 454,
    SupplyNotFound = 455,
}
