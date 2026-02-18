#![no_std]

mod contract;
pub mod errors;
mod events;
mod fee;
mod index;
mod interface;
mod oracle;
mod shares;
mod storage;
#[cfg(test)]
mod test_rebalance;
#[cfg(test)]
mod test_refactor;
mod volume;

#[cfg(test)]
mod test_utils;

// mod test;
// mod test_math;
// mod test_permissions;
// mod testutils;

pub use crate::contract::{IndexFund, IndexFundClient};
