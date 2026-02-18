#![no_std]

mod contract;
mod normal_treasury;
mod storage;
mod types;

pub use crate::contract::{NormalAdapter, NormalAdapterClient};
