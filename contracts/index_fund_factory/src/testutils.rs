#![allow(dead_code)]
#![cfg(test)]
extern crate std;

use crate::{storage::IndexFundFactoryConfig, IndexFundFactoryClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};
use std::vec;
use token_share::token_contract::{Client as IndexTokenClient, WASM as IndexTokenWASM};

pub mod index_fund {
    soroban_sdk::contractimport!(file = "../../wasm/index_fund.wasm");
}

pub fn install_index_hash(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(index_fund::WASM)
}

pub fn install_token_wasm(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(IndexTokenWASM)
}

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

    pub(crate) factory: IndexFundFactoryClient<'a>,
    pub(crate) adapter_registry: Address,

    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) rewards_admin: Address,
    pub(crate) operations_admin: Address,
    pub(crate) fee_admin: Address,
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
        let rewards_admin = Address::generate(&e);
        let operations_admin = Address::generate(&e);
        let fee_admin = Address::generate(&e);

        let index_hash = install_index_hash(&e);
        let adapter_registry = Address::generate(&e);

        let factory = create_factory_contract(
            &e,
            &admin,
            &emergency_admin,
            &rewards_admin,
            &operations_admin,
            &fee_admin,
            &(IndexFundFactoryConfig {
                index_contract_wasm: install_index_hash(&e),
                index_token_wasm: install_token_wasm(&e),
                adapter_registry: adapter_registry.clone(),
            }),
        );

        Self {
            env: e,
            users,
            factory,
            admin,
            emergency_admin,
            rewards_admin,
            operations_admin,
            fee_admin,

            adapter_registry,
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

pub fn create_factory_contract<'a>(
    e: &Env,
    admin: &Address,
    emergency_admin: &Address,
    rewards_admin: &Address,
    operations_admin: &Address,
    fee_admin: &Address,
    config: &IndexFundFactoryConfig,
) -> IndexFundFactoryClient<'a> {
    let factory = IndexFundFactoryClient::new(
        e,
        &e.register(
            crate::IndexFundFactory {},
            (
                admin,
                emergency_admin,
                rewards_admin,
                operations_admin,
                fee_admin,
                config.clone(),
            ),
        ),
    );
    factory
}
