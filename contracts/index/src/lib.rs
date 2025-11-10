#![no_std]

mod contract;
pub mod errors;
mod events;
mod fees;
mod index;
mod interface;
mod storage;
mod volume;
#[cfg(test)]
mod test_fees;
// mod test;
// mod test_math;
// mod test_permissions;
// mod testutils;

pub use crate::contract::{Index, IndexClient};
