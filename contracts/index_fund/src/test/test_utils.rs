#![cfg(test)]

use crate::{
    interface::QueryInterface,
    storage::{set_component_balance, set_swap_utility, set_token_quote},
};
use access_control::access::AccessControl;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::Role;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env, Symbol, Vec};
use types::{adapter::AdapterTradeParams, component::Component};

//TODO: Remove completely mock swaps from here and use the swap utility contract for testing

#[contract]
pub struct MockAdapter;

#[contractimpl]
impl MockAdapter {
    pub fn buy(_env: Env, params: AdapterTradeParams) -> u128 {
        params.amount_in
    }

    pub fn sell(_env: Env, params: AdapterTradeParams) -> u128 {
        params.amount_in
    }

    pub fn initialize(
        _env: Env,
        _admin: Address,
        _normal_dex_address: Address,
        _soroswap_address: Address,
        _xlm_token_address: Address,
    ) {
    }

    pub fn is_initialized(_env: Env) -> bool {
        true
    }
}

#[contract]
pub struct MockFactory;

#[contractimpl]
impl MockFactory {
    pub fn get_swap_utility(env: Env) -> Address {
        let key = Symbol::new(&env, "swap_util");
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Address::generate(&env))
    }

    pub fn set_swap_utility(env: Env, swap_utility: Address) {
        let key = Symbol::new(&env, "swap_util");
        env.storage().instance().set(&key, &swap_utility);
    }

    // Mock price conversion - returns the amount as-is (1:1 conversion for testing)
    pub fn convert_token_to_usd_safe(_env: Env, _token: Address, amount: u128) -> Option<u128> {
        // Return mock price: 1 token = 1 USD (with 7 decimals)
        Some(amount)
    }

    pub fn convert_token_to_usd(_env: Env, _token: Address, amount: u128) -> u128 {
        // Return mock price: 1 token = 1 USD
        amount
    }
}

pub fn create_mock_swap_utility(e: &Env) -> Address {
    e.register(MockAdapter, ())
}

pub fn create_mock_factory(e: &Env) -> Address {
    e.register(MockFactory, ())
}

pub fn setup_test_contracts(e: &Env) -> (Address, Address, Address, Address) {
    let index_contract = e.register(crate::contract::IndexFund, ());

    let admin = Address::generate(e);
    let token = Address::generate(e);
    let swap_utility = create_mock_swap_utility(e);

    // Create mock tokens for token_quote and token_share
    let token_quote = Address::generate(e);
    let token_share = Address::generate(e);

    let client = crate::contract::IndexFundClient::new(e, &index_contract);
    // client.initialize(&admin, &token);

    e.as_contract(&index_contract, || {
        set_swap_utility(e, &swap_utility);
        // crate::storage::set_adapter_for_type(e, Symbol::new(e, "Normal"), &swap_utility);
        // crate::storage::set_adapter_for_type(e, AdapterType::Aquarius, &swap_utility);
        // crate::storage::set_adapter_for_type(e, AdapterType::Soroswap, &swap_utility);

        // Set up admin role in AccessControl - required for permission checks
        let access_control = AccessControl::new(e);
        access_control.set_role_address(&Role::Admin, &admin);

        // Set up token_quote - required for mint operations
        set_token_quote(e, &token_quote);

        // Set up token_share - required for share operations
        token_share::put_token_share(e, token_share);
    });

    (index_contract, admin, token, swap_utility)
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn balance(env: Env, id: Address) -> i128 {
        // Return a large balance for all accounts to simulate having tokens
        1_000_000_000 // 1B tokens
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        // Mock implementation - just verify auth and succeed
        from.require_auth();
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        // Mock implementation - just verify auth and succeed
        spender.require_auth();
    }

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        // Mock implementation - just verify auth
        from.require_auth();
    }

    pub fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        i128::MAX // Allow unlimited allowance
    }

    // Admin functions for token minting/burning
    pub fn mint(env: Env, to: Address, amount: i128) {
        // Mock implementation - just succeed
        // In a real token, this would create new tokens
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        // Mock implementation - just succeed
        // In a real token, this would destroy tokens
        from.require_auth();
    }
}

pub fn create_mock_token(e: &Env) -> Address {
    e.register(MockToken, ())
}

pub fn setup_components_with_balances(
    e: &Env,
    contract: &Address,
    tokens_with_weights_and_balances: Vec<(Address, u128, u128)>,
) {
    e.as_contract(contract, || {
        for (token, weight, balance) in tokens_with_weights_and_balances.iter() {
            let oracle = Address::generate(e);
            let component = Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
                oracle,
                adapter: Symbol::new(e, "Normal"),
            };
            crate::storage::set_component(e, token.clone(), component);
            crate::storage::add_component_to_registry(e, token.clone());

            set_component_balance(e, token.clone(), balance);
        }
    });
}

pub fn enhanced_setup_components(
    e: &Env,
    contract: &Address,
    tokens_with_weights: Vec<(Address, u128)>,
) {
    e.as_contract(contract, || {
        let current_nav = crate::IndexFund::get_current_nav(e.clone());

        for (token, weight) in tokens_with_weights.iter() {
            let oracle = Address::generate(e);
            let component = Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
                oracle,
                adapter: Symbol::new(e, "Normal"),
            };
            crate::storage::set_component(e, token.clone(), component);
            crate::storage::add_component_to_registry(e, token.clone());

            let target_balance = (current_nav * weight) / 10000;
            set_component_balance(e, token.clone(), target_balance);
        }
    });
}

pub fn setup_components_without_balances(
    e: &Env,
    contract: &Address,
    tokens_with_weights: Vec<(Address, u128)>,
) {
    e.as_contract(contract, || {
        for (token, weight) in tokens_with_weights.iter() {
            let oracle = Address::generate(e);
            let component = Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
                oracle,
                adapter: Symbol::new(e, "Normal"),
            };
            crate::storage::set_component(e, token.clone(), component);
            crate::storage::add_component_to_registry(e, token.clone());
        }
    });
}

pub fn setup_components_with_zero_balances(
    e: &Env,
    contract: &Address,
    tokens_with_weights: Vec<(Address, u128)>,
) {
    e.as_contract(contract, || {
        for (token, weight) in tokens_with_weights.iter() {
            let oracle = Address::generate(e);
            let component = Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
                oracle,
                adapter: Symbol::new(e, "Normal"),
            };
            crate::storage::set_component(e, token.clone(), component);
            crate::storage::add_component_to_registry(e, token.clone());

            set_component_balance(e, token.clone(), 0);
        }
    });
}

pub fn init_mock_factory(e: &Env, factory_address: &Address, swap_utility: &Address) {
    let factory_client = MockFactoryClient::new(e, factory_address);
    factory_client.set_swap_utility(swap_utility);
}

pub fn complete_test_setup(e: &Env) -> (Address, Address, Address, Address, Address) {
    let (index_contract, admin, token, swap_utility) = setup_test_contracts(e);

    let factory = create_mock_factory(e);
    init_mock_factory(e, &factory, &swap_utility);

    e.as_contract(&index_contract, || {
        crate::storage::set_factory(e, &factory);
    });

    (index_contract, admin, token, swap_utility, factory)
}

pub fn create_balanced_test_scenario(
    e: &Env,
    contract: &Address,
    num_tokens: u32,
    total_nav: u128,
) -> Vec<Address> {
    let mut tokens = Vec::new(e);
    let weight_per_token = 10000 / (num_tokens as u128);
    let balance_per_token = total_nav / (num_tokens as u128);

    for _ in 0..num_tokens {
        let token = create_mock_token(e);
        tokens.push_back(token.clone());

        e.as_contract(contract, || {
            let oracle = Address::generate(e);
            let component = Component {
                asset: Symbol::new(e, "TOKEN"),
                weight: weight_per_token,
                oracle,
                adapter: Symbol::new(e, "Normal"),
            };
            crate::storage::set_component(e, token.clone(), component);
            crate::storage::add_component_to_registry(e, token.clone());
            set_component_balance(e, token, balance_per_token);
        });
    }

    // e.as_contract(contract, || {
    //     set_base_nav(e, &total_nav);
    // });

    tokens
}

pub fn setup_mock_token_shares(e: &Env, contract: &Address, total_shares: u128) {
    e.as_contract(contract, || {
        token_share::put_total_shares(e, total_shares);
    });
}

pub fn create_test_index_with_valid_addresses(e: &Env) -> (Address, Address, Address) {
    let (contract_address, admin, token, _swap_utility, _factory) = complete_test_setup(e);

    (contract_address, admin, token)
}
