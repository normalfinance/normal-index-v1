#![no_std]

mod contract;
mod soroswap_router;
mod storage;
mod types;

pub use crate::contract::{SoroswapAdapter, SoroswapAdapterClient};
