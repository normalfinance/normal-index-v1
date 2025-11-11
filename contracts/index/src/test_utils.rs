#![cfg(test)]

use soroban_sdk::{testutils::Address as _, contract, contractimpl, Address, Env, Symbol, Vec};
use crate::storage::{set_swap_utility, set_component_balance, set_base_nav};


use crate::index::{SwapUtilityParams, SwapResult, DexProvider};


//TODO: Remove completely mock swaps from here and use the swap utility contract for testing


#[contract]
pub struct MockSwapUtility;

#[contractimpl]
impl MockSwapUtility {
    
    pub fn execute_swap(
        _env: Env,
        params: SwapUtilityParams,
    ) -> SwapResult {
        
        SwapResult {
            provider_used: DexProvider::Normal,
            amount_in: params.amount_in as u128,
            amount_out: params.amount_in as u128, 
            success: true,
        }
    }

    
    pub fn execute_batch_swaps(
        env: Env,
        swaps: Vec<SwapUtilityParams>,
    ) -> Vec<SwapResult> {
        let mut results = Vec::new(&env);
        
        for i in 0..swaps.len() {
            let swap_params = swaps.get(i).unwrap();
            let result = Self::execute_swap(env.clone(), swap_params);
            results.push_back(result);
        }
        
        results
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
        env.storage().instance().get(&key).unwrap_or_else(|| Address::generate(&env))
    }
    
    pub fn get_fee_enabled(_env: Env) -> bool {
        false 
    }

    pub fn set_swap_utility(env: Env, swap_utility: Address) {
        let key = Symbol::new(&env, "swap_util");
        env.storage().instance().set(&key, &swap_utility);
    }
}


pub fn create_mock_swap_utility(e: &Env) -> Address {
    e.register(MockSwapUtility, ())
}


pub fn create_mock_factory(e: &Env) -> Address {
    e.register(MockFactory, ())
}


pub fn setup_test_contracts(e: &Env) -> (Address, Address, Address, Address) {
    
    let index_contract = e.register(crate::contract::Index, ());
    
    
    let admin = Address::generate(e);
    let token = Address::generate(e);
    let swap_utility = create_mock_swap_utility(e);
    
    
    let client = crate::contract::IndexClient::new(e, &index_contract);
    client.initialize(&admin, &token);
    
    
    e.as_contract(&index_contract, || {
        set_swap_utility(e, &swap_utility);
        
        set_base_nav(e, &100_000);
    });
    
    (index_contract, admin, token, swap_utility)
}


pub fn create_mock_token(e: &Env) -> Address {
    Address::generate(e)
}


pub fn setup_components_with_balances(
    e: &Env, 
    contract: &Address, 
    tokens_with_weights_and_balances: Vec<(Address, u128, u128)>
) {
    e.as_contract(contract, || {
        for (token, weight, balance) in tokens_with_weights_and_balances.iter() {
            let component = crate::storage::Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
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
    tokens_with_weights: Vec<(Address, u128)>
) {
    e.as_contract(contract, || {
        let base_nav = crate::storage::get_base_nav(e) as u128;
        
        for (token, weight) in tokens_with_weights.iter() {
            let component = crate::storage::Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
            };
            crate::storage::set_component(e, token.clone(), component);
            crate::storage::add_component_to_registry(e, token.clone());
            
            
            let target_balance = (base_nav * weight) / 10000;
            set_component_balance(e, token.clone(), target_balance);
        }
    });
}


pub fn setup_components_without_balances(
    e: &Env, 
    contract: &Address, 
    tokens_with_weights: Vec<(Address, u128)>
) {
    e.as_contract(contract, || {
        for (token, weight) in tokens_with_weights.iter() {
            let component = crate::storage::Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
            };
            crate::storage::set_component(e, token.clone(), component);
            crate::storage::add_component_to_registry(e, token.clone());
            
        }
    });
}


pub fn setup_components_with_zero_balances(
    e: &Env, 
    contract: &Address, 
    tokens_with_weights: Vec<(Address, u128)>
) {
    e.as_contract(contract, || {
        for (token, weight) in tokens_with_weights.iter() {
            let component = crate::storage::Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
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
    let weight_per_token = 10000 / num_tokens as u128;
    let balance_per_token = total_nav / num_tokens as u128;
    
    for _ in 0..num_tokens {
        let token = create_mock_token(e);
        tokens.push_back(token.clone());
        
        e.as_contract(contract, || {
            let component = crate::storage::Component {
                asset: Symbol::new(e, "TOKEN"),
                weight: weight_per_token,
            };
            crate::storage::set_component(e, token.clone(), component);
            crate::storage::add_component_to_registry(e, token.clone());
            set_component_balance(e, token, balance_per_token);
        });
    }
    
    
    e.as_contract(contract, || {
        set_base_nav(e, &total_nav);
    });
    
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