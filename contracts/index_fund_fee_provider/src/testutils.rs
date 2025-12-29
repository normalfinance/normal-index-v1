#![allow(dead_code)]
#![cfg(test)]
extern crate std;

use crate::IndexFundFeeProviderClient;
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::BytesN;
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol, Vec};
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
    pub(crate) provider: IndexFundFeeProviderClient<'a>,
    pub(crate) factory: Address,
    pub(crate) index_fund: Address,
    pub(crate) fee_destination: Address,

    pub(crate) token_usdc: SorobanTokenClient<'a>,
    pub(crate) token_usdc_admin_client: SorobanTokenAdminClient<'a>,

    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) fee_admin: Address,
    pub(crate) pause_admin: Address,
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
        let fee_admin = Address::generate(&e);
        let pause_admin = Address::generate(&e);

        let fee_destination = Address::generate(&e);

        let token_usdc = create_token_contract(&e, &admin);
        let token_usdc_admin_client = get_token_admin_client(&e, &token_usdc.address.clone());

        let provider = create_provider_contract(
            &e,
            &admin,
            &(admin.clone(), admin.clone(), admin.clone()),
            &token_usdc.address,
        );

        // create index fund
        let factory = create_factory_contract(&admin);

        let index_fund = factory.deploy_index_fund();

        Self {
            env: e,
            users,
            provider,
            fee_destination,
            factory,
            index_fund,
            token_usdc,
            token_usdc_admin_client,
            admin,
            emergency_admin,
            fee_admin,
            pause_admin,
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

pub fn create_provider_contract<'a>(
    e: &Env,
    admin: &Address,
    privileged_addrs: &(Address, Address, Address),
    fee_token: &Address,
) -> IndexFundFeeProviderClient<'a> {
    let provider = IndexFundFeeProviderClient::new(
        e,
        &e.register(
            crate::IndexFundFeeProvider {},
            (admin, privileged_addrs, fee_token),
        ),
    );
    provider
}

pub(crate) fn create_token_contract<'a>(e: &Env, admin: &Address) -> SorobanTokenClient<'a> {
    SorobanTokenClient::new(
        e,
        &e.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

pub(crate) fn get_token_admin_client<'a>(
    e: &Env,
    address: &Address,
) -> SorobanTokenAdminClient<'a> {
    SorobanTokenAdminClient::new(e, address)
}

fn install_token_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../wasm/soroban_token_contract.wasm");
    e.deployer().upload_contract_wasm(WASM)
}
