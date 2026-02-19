#![no_std]

mod aquarius_router;
mod contract;
mod storage;
mod types;

mod test;

pub use crate::contract::{AquariusAdapter, AquariusAdapterClient};
