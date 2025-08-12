#![no_std]

mod contract;
mod events;
mod index_utils;
mod interface;
mod storage;
// mod test;
// mod test_permissions;
// mod testutils;

pub use crate::contract::{IndexFactory, IndexFactoryClient};
