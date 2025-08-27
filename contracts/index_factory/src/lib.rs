#![no_std]

mod contract;
mod events;
mod index_utils;
mod interface;
mod oracle;
mod storage;
mod tiers;
// mod test;
// mod test_permissions;
// mod testutils;

pub use crate::contract::{IndexFactory, IndexFactoryClient};
