#![allow(dead_code)]
#![cfg(test)]
extern crate std;

use crate::AdapterRegistryClient;
use soroban_sdk::{testutils::Address as _, Address, Env};
use std::vec;

pub(crate) struct TestConfig {
    pub(crate) users_count: u32,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig { users_count: 2 }
    }
}

pub(crate) struct Setup<'a> {
    pub(crate) env: Env,
    pub(crate) users: vec::Vec<Address>,

    pub(crate) registry: AdapterRegistryClient<'a>,

    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) operations_admin: Address,
}

impl Default for Setup<'_> {
    // Create setup from default config
    fn default() -> Self {
        let default_config = TestConfig::default();
        Self::new_with_config(&default_config)
    }
}

impl Setup<'_> {
    // Create setup from config
    pub(crate) fn new_with_config(config: &TestConfig) -> Self {
        let setup = Self::setup(config);
        setup
    }

    pub(crate) fn setup(config: &TestConfig) -> Self {
        let e: Env = Env::default();
        e.mock_all_auths();
        e.cost_estimate().budget().reset_unlimited();

        let users = Self::generate_random_users(&e, config.users_count);
        let admin = users[0].clone();
        let emergency_admin = Address::generate(&e);
        let operations_admin = Address::generate(&e);

        let registry = create_registry_contract(&e, &admin, &emergency_admin, &operations_admin);

        Self {
            env: e,
            users,
            registry,
            admin,
            emergency_admin,
            operations_admin,
        }
    }

    pub(crate) fn generate_random_users(e: &Env, users_count: u32) -> vec::Vec<Address> {
        let mut users = vec![];
        for _c in 0..users_count {
            users.push(Address::generate(e));
        }
        users
    }
}

pub fn create_registry_contract<'a>(
    e: &Env,
    admin: &Address,
    emergency_admin: &Address,
    operations_admin: &Address,
) -> AdapterRegistryClient<'a> {
    let factory = AdapterRegistryClient::new(
        e,
        &e.register(
            crate::AdapterRegistry {},
            (admin, emergency_admin, operations_admin),
        ),
    );
    factory
}
