#![no_std]

mod contract;
pub mod errors;
mod events;
mod interface;
mod storage;
mod test;
mod test_permissions;
mod testutils;

pub use crate::contract::{IndexFundFeeProvider, IndexFundFeeProviderClient};
