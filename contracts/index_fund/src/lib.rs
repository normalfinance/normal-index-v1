#![no_std]

mod adapter;
mod contract;
pub mod errors;
mod events;
mod fee;
mod index;
mod interface;
mod oracle;
mod rebalance;
mod refactor;
mod shares;
mod storage;
pub mod token;
mod volume;

mod test;

pub use crate::contract::{IndexFund, IndexFundClient};
