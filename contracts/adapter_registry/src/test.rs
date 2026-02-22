#![cfg(test)]
extern crate std;

use crate::testutils;
use crate::testutils::Setup;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, String, Symbol, Vec};

#[test]
fn test_set_adapter() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let usdc = Address::generate(&setup.env);

    // TODO:
    // assert_eq!(index_client.get_factory(), setup.factory.address);
}
