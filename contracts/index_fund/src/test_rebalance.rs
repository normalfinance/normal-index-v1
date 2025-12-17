#![cfg(test)]

use super::contract::{IndexFund, IndexFundClient};
use super::interface::RebalanceParams;
use super::storage::{
    add_component_to_registry, get_component, get_last_rebalance_ts, get_last_updated_ts,
    get_rebalance_authority_status, get_rebalance_threshold, set_base_nav, set_component,
    set_component_balance, set_last_rebalance_ts, set_last_updated_ts, set_public,
    set_rebalance_threshold, set_swap_utility, Component,
};
use super::test_utils::{
    complete_test_setup, create_mock_token, enhanced_setup_components,
    setup_components_with_zero_balances, setup_components_without_balances,
    setup_mock_token_shares,
};
use soroban_sdk::{log, testutils::Address as _, vec, Address, Env, Symbol, Vec};
use utils::test_utils::jump;

const THIRTY_DAYS: u64 = 30 * 24 * 60 * 60;

// Test utilities

fn register_test_contract(e: &Env) -> Address {
    e.register(IndexFund, ())
}

fn create_test_index(e: &Env) -> (Address, Address, Address) {
    let (contract_address, admin, token, _swap_utility, _factory) = complete_test_setup(e);

    // Set additional test configuration
    e.as_contract(&contract_address, || {
        // Set default rebalance threshold
        set_rebalance_threshold(e, &THIRTY_DAYS);
    });

    (contract_address, admin, token)
}

fn setup_components(e: &Env, contract: &Address, tokens_with_weights: Vec<(Address, u128)>) {
    enhanced_setup_components(e, contract, tokens_with_weights);
}

// Helper function to make rebalance immediately allowed by setting an old timestamp
fn allow_immediate_rebalance(e: &Env, contract: &Address) {
    e.as_contract(contract, || {
        let current_time = e.ledger().timestamp();
        // Set last rebalance to well in the past (more than THIRTY_DAYS ago)
        if current_time > THIRTY_DAYS {
            set_last_rebalance_ts(e, &(current_time - THIRTY_DAYS - 1));
        } else {
            // If current time is less than THIRTY_DAYS, set to 0 and advance time
            set_last_rebalance_ts(e, &0);
        }
    });

    // Advance time past the threshold
    jump(e, THIRTY_DAYS + 1);
}

// ===== Basic Rebalance Operations =====

#[test]
fn test_rebalance_updates_timestamps() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    let time_before = e.ledger().timestamp();

    // Execute rebalance
    let params = RebalanceParams { target_nav: None };
    client.rebalance(&admin, &params);

    // Verify both timestamps updated
    let last_rebalance = e.as_contract(&contract_address, || get_last_rebalance_ts(&e));
    let last_updated = e.as_contract(&contract_address, || get_last_updated_ts(&e));

    assert!(
        last_rebalance >= time_before,
        "last_rebalance_ts should be updated"
    );
    assert!(
        last_updated >= time_before,
        "last_updated_ts should be updated"
    );
    assert_eq!(
        last_rebalance, last_updated,
        "Both timestamps should be equal after rebalance"
    );
}

#[test]
fn test_rebalance_with_target_nav() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 10000)]);

    // Set base NAV
    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        set_component_balance(&e, token, 50_000);
    });

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Execute rebalance with specific target_nav
    let params = RebalanceParams {
        target_nav: Some(200_000),
    };
    client.rebalance(&admin, &params);

    // Should succeed - swap generation will use target_nav
}

#[test]
fn test_rebalance_without_target_nav_uses_current() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 10000)]);

    // Set base NAV
    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        set_component_balance(&e, token, 100_000);
    });

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Execute rebalance without target_nav (None)
    let params = RebalanceParams { target_nav: None };
    client.rebalance(&admin, &params);

    // Should succeed using current NAV
}

// ===== Time Threshold Enforcement =====

#[test]
#[should_panic(expected = "Error(Contract, #37)")]
fn test_rebalance_too_soon_fails() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance for first call
    allow_immediate_rebalance(&e, &contract_address);

    // First rebalance
    let params = RebalanceParams { target_nav: None };
    client.rebalance(&admin, &params);

    // Try to rebalance immediately (threshold - 1 second)
    let threshold = e.as_contract(&contract_address, || get_rebalance_threshold(&e));
    jump(&e, threshold - 1);

    // Should fail
    client.rebalance(&admin, &params);
}

#[test]
fn test_rebalance_after_threshold_succeeds() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance for first call
    allow_immediate_rebalance(&e, &contract_address);

    // First rebalance
    let params = RebalanceParams { target_nav: None };
    client.rebalance(&admin, &params);

    // Wait for threshold period
    let threshold = e.as_contract(&contract_address, || get_rebalance_threshold(&e));
    jump(&e, threshold);

    // Second rebalance should succeed
    client.rebalance(&admin, &params);
}

#[test]
fn test_get_rebalance_status_timing() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Execute first rebalance
    let params = RebalanceParams { target_nav: None };
    client.rebalance(&admin, &params);

    // Query status before threshold
    let status_before = client.get_rebalance_status();
    assert!(
        !status_before.can_rebalance,
        "Should not be able to rebalance yet"
    );
    assert!(
        status_before.time_until_next_rebalance > 0,
        "Should have time remaining"
    );

    // Jump past threshold
    let threshold = e.as_contract(&contract_address, || get_rebalance_threshold(&e));
    jump(&e, threshold);

    // Query status after threshold
    let status_after = client.get_rebalance_status();
    assert!(
        status_after.can_rebalance,
        "Should be able to rebalance now"
    );
    assert_eq!(
        status_after.time_until_next_rebalance, 0,
        "No time should remain"
    );
}

#[test]
fn test_custom_rebalance_threshold() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    // Set custom threshold (7 days)
    let custom_threshold = 7 * 24 * 60 * 60;
    client.set_rebalance_threshold(&admin, &custom_threshold);

    // Verify threshold updated
    let threshold = e.as_contract(&contract_address, || get_rebalance_threshold(&e));
    assert_eq!(threshold, custom_threshold);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Execute first rebalance
    client.rebalance(&admin, &RebalanceParams { target_nav: None });

    // Jump less than custom threshold
    jump(&e, custom_threshold - 1);

    // Check that can_rebalance returns false
    let status = client.get_rebalance_status();
    assert!(
        !status.can_rebalance,
        "Should not be able to rebalance before custom threshold"
    );
}

// ===== Permission Checks =====

#[test]
#[should_panic(expected = "Error(Contract, #39)")]
fn test_public_index_rebalance_requires_admin() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, _, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    // Set as public index
    e.as_contract(&contract_address, || {
        set_public(&e, &true);
    });

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    let non_admin = Address::generate(&e);

    // Non-admin tries to rebalance public index
    client.rebalance(&non_admin, &RebalanceParams { target_nav: None });
}

#[test]
fn test_private_index_admin_can_rebalance() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Admin rebalances private index - should succeed
    client.rebalance(&admin, &RebalanceParams { target_nav: None });
}

#[test]
fn test_private_index_rebalance_authority() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let authority = Address::generate(&e);

    // Set as rebalance authority
    client.set_rebalance_authority(&admin, &authority, &true);

    // Verify authority status
    let has_authority = e.as_contract(&contract_address, || {
        get_rebalance_authority_status(&e, &authority)
    });
    assert!(has_authority);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Authority rebalances - should succeed
    client.rebalance(&authority, &RebalanceParams { target_nav: None });
}

#[test]
#[should_panic(expected = "Error(Contract, #43)")]
fn test_unauthorized_cannot_rebalance_private_index() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, _, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    let unauthorized_user = Address::generate(&e);

    // Unauthorized user tries to rebalance - should fail
    client.rebalance(&unauthorized_user, &RebalanceParams { target_nav: None });
}

#[test]
fn test_set_rebalance_authority() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let authority1 = Address::generate(&e);
    let authority2 = Address::generate(&e);

    // Add first authority
    client.set_rebalance_authority(&admin, &authority1, &true);

    // Verify in authorities list
    let authorities = client.get_rebalance_authorities();
    assert_eq!(authorities.len(), 1);

    // Add second authority
    client.set_rebalance_authority(&admin, &authority2, &true);

    let authorities = client.get_rebalance_authorities();
    assert_eq!(authorities.len(), 2);

    // Remove first authority
    client.set_rebalance_authority(&admin, &authority1, &false);

    let authorities = client.get_rebalance_authorities();
    assert_eq!(authorities.len(), 1);

    // Verify authority1 removed
    let has_authority1 = e.as_contract(&contract_address, || {
        get_rebalance_authority_status(&e, &authority1)
    });
    assert!(!has_authority1);

    // Verify authority2 still has authority
    let has_authority2 = e.as_contract(&contract_address, || {
        get_rebalance_authority_status(&e, &authority2)
    });
    assert!(has_authority2);
}

// ===== Swap Generation Logic =====

#[test]
fn test_generate_rebalance_swaps_buy() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);

    // Component weight 50% (5000), current balance 30% of target NAV
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 5000)]);

    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        set_component_balance(&e, token.clone(), 30_000); // 30% of NAV
    });

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Rebalance should generate buy swap for difference
    client.rebalance(&admin, &RebalanceParams { target_nav: None });

    // Component balance should be adjusted (or swaps generated)
}

#[test]
fn test_generate_rebalance_swaps_sell() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);

    // Component weight 30% (3000), current balance 50% of target NAV
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 3000)]);

    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        set_component_balance(&e, token, 50_000); // 50% of NAV
    });

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Rebalance should generate sell swap for difference
    client.rebalance(&admin, &RebalanceParams { target_nav: None });
}

#[test]
fn test_generate_rebalance_swaps_no_change() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);

    // Component weight 50% (5000), current balance exactly 50% of NAV
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 5000)]);

    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        set_component_balance(&e, token, 50_000); // Exactly 50%
    });

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Rebalance should generate no swaps
    client.rebalance(&admin, &RebalanceParams { target_nav: None });
}

#[test]
fn test_generate_rebalance_swaps_multiple_components() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token1 = create_mock_token(&e);
    let token2 = create_mock_token(&e);
    let token3 = create_mock_token(&e);

    // Three components with different weights
    setup_components(
        &e,
        &contract_address,
        vec![
            &e,
            (token1.clone(), 4000), // 40%
            (token2.clone(), 3500), // 35%
            (token3.clone(), 2500), // 25%
        ],
    );

    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        set_component_balance(&e, token1, 30_000); // Underweight - need to buy
        set_component_balance(&e, token2, 35_000); // Exactly right
        set_component_balance(&e, token3, 35_000); // Overweight - need to sell
    });

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Rebalance should generate buy for token1, sell for token3, nothing for token2
    client.rebalance(&admin, &RebalanceParams { target_nav: None });
}

// // ===== Kill Switch =====

// #[test]
// #[should_panic(expected = "Error(Contract, #32)")]
// fn test_rebalance_killed_prevents_rebalance() {
//     let e = Env::default();
//     e.mock_all_auths();

//     let (contract_address, admin, _) = create_test_index(&e);
//     let client = IndexFundClient::new(&e, &contract_address);

//     let token = create_mock_token(&e);
//     setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

//     // Allow immediate rebalance (setup timing)
//     allow_immediate_rebalance(&e, &contract_address);

//     // Kill rebalance
//     client.kill_rebalance(&admin);

//     // Attempt rebalance - should fail
//     client.rebalance(&admin, &RebalanceParams { target_nav: None });
// }

// #[test]
// fn test_unkill_rebalance_restores_functionality() {
//     let e = Env::default();
//     e.mock_all_auths();

//     let (contract_address, admin, _) = create_test_index(&e);
//     let client = IndexFundClient::new(&e, &contract_address);

//     let token = create_mock_token(&e);
//     setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

//     // Allow immediate rebalance
//     allow_immediate_rebalance(&e, &contract_address);

//     // Kill then unkill
//     client.kill_rebalance(&admin);
//     client.unkill_rebalance(&admin);

//     // Rebalance should succeed
//     client.rebalance(&admin, &RebalanceParams { target_nav: None });
// }

// ===== Query Functions =====

#[test]
fn test_can_address_rebalance_admin() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance by setting time threshold
    allow_immediate_rebalance(&e, &contract_address);

    // Admin should be able to rebalance
    let can_rebalance = client.can_address_rebalance(&admin);
    assert!(can_rebalance, "Admin should be able to rebalance");
}

#[test]
fn test_can_address_rebalance_authority() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let authority = Address::generate(&e);

    // Set as rebalance authority
    client.set_rebalance_authority(&admin, &authority, &true);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance by setting time threshold
    allow_immediate_rebalance(&e, &contract_address);

    // Authority should be able to rebalance
    let can_rebalance = client.can_address_rebalance(&authority);
    assert!(can_rebalance, "Authority should be able to rebalance");
}

#[test]
fn test_can_address_rebalance_regular_user() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, _, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let regular_user = Address::generate(&e);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Regular user should NOT be able to rebalance
    let can_rebalance = client.can_address_rebalance(&regular_user);
    assert!(
        !can_rebalance,
        "Regular user should not be able to rebalance"
    );
}

#[test]
fn test_get_component_allocation() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, _, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token1 = create_mock_token(&e);
    let token2 = create_mock_token(&e);

    // Setup components with weights only (no auto-balances)
    setup_components_without_balances(
        &e,
        &contract_address,
        vec![&e, (token1.clone(), 6000), (token2.clone(), 4000)],
    );

    // Set balances different from target weights and setup token shares
    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        set_component_balance(&e, token1.clone(), 50_000); // Target should be 60_000
        set_component_balance(&e, token2.clone(), 50_000); // Target should be 40_000
    });

    // Setup mock token shares for NAV calculation
    setup_mock_token_shares(&e, &contract_address, 1_000_000); // 1M total shares

    // Get component allocation
    let allocations = client.get_component_allocation();

    assert_eq!(allocations.len(), 2);

    // Verify token1 allocation
    let alloc1 = allocations.get(token1).unwrap();
    assert_eq!(alloc1.current_balance, 50_000);
    assert_eq!(alloc1.target_balance, 60_000);

    // Verify token2 allocation
    let alloc2 = allocations.get(token2).unwrap();
    assert_eq!(alloc2.current_balance, 50_000);
    assert_eq!(alloc2.target_balance, 40_000);
}

// ===== Integration Tests =====

#[test]
fn test_full_refactor_rebalance_flow() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token1 = create_mock_token(&e);
    let token2 = create_mock_token(&e);
    let token3 = create_mock_token(&e);

    // Initial state: 2 components (50%/50%) - setup without automatic balances
    setup_components_without_balances(
        &e,
        &contract_address,
        vec![&e, (token1.clone(), 5000), (token2.clone(), 5000)],
    );

    // Set base NAV and initial component balances manually
    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        // Set balanced initial state
        set_component_balance(&e, token1.clone(), 50_000); // 50% of NAV
        set_component_balance(&e, token2.clone(), 50_000); // 50% of NAV
    });

    // Set initial rebalance timestamp
    e.as_contract(&contract_address, || {
        set_last_rebalance_ts(&e, &100);
        set_last_updated_ts(&e, &100);
    });

    // Advance time to ensure refactor gets a different timestamp (105 > 100)
    jump(&e, 105);

    // Refactor to 3 components (40%/30%/30%)
    use super::interface::{ComponentAction, ComponentUpdate, RefactorParams};
    let refactor_updates = vec![
        &e,
        ComponentUpdate {
            token: token1.clone(),
            new_weight: 4000,
            action: ComponentAction::UpdateWeight,
        },
        ComponentUpdate {
            token: token2.clone(),
            new_weight: 3000,
            action: ComponentAction::UpdateWeight,
        },
        ComponentUpdate {
            token: token3.clone(),
            new_weight: 3000,
            action: ComponentAction::Add,
        },
    ];

    client.refactor(
        &admin,
        &RefactorParams {
            component_updates: refactor_updates,
        },
    );

    // Verify refactor updated last_updated_ts but not last_rebalance_ts
    let last_updated = e.as_contract(&contract_address, || get_last_updated_ts(&e));
    let last_rebalance = e.as_contract(&contract_address, || get_last_rebalance_ts(&e));
    assert!(last_updated > last_rebalance);

    // Mint is now allowed after refactor (check removed - see test_mint_allowed_after_refactor)

    // Create imbalanced component allocations to force swap generation
    // Target weights after refactor: token1=40%, token2=30%, token3=30%
    // Set imbalanced state: token1=60%, token2=20%, token3=20%
    e.as_contract(&contract_address, || {
        set_component_balance(&e, token1.clone(), 60_000); // 60% instead of target 40%
        set_component_balance(&e, token2.clone(), 20_000); // 20% instead of target 30%
        set_component_balance(&e, token3.clone(), 20_000); // 20% instead of target 30%
    });

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Rebalance to align balances with new weights
    client.rebalance(&admin, &RebalanceParams { target_nav: None });

    // Verify timestamps now match
    let last_updated_after = e.as_contract(&contract_address, || get_last_updated_ts(&e));
    let last_rebalance_after = e.as_contract(&contract_address, || get_last_rebalance_ts(&e));
    assert_eq!(last_updated_after, last_rebalance_after);

    // Verify proper weight distribution
    let comp1 = e.as_contract(&contract_address, || get_component(&e, token1));
    let comp2 = e.as_contract(&contract_address, || get_component(&e, token2));
    let comp3 = e.as_contract(&contract_address, || get_component(&e, token3));

    assert_eq!(comp1.weight, 4000);
    assert_eq!(comp2.weight, 3000);
    assert_eq!(comp3.weight, 3000);
    assert_eq!(comp1.weight + comp2.weight + comp3.weight, 10000);
}

#[test]
fn test_rebalance_after_initial_setup() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Initial state: last_rebalance_ts = 0
    let last_rebalance_before = e.as_contract(&contract_address, || get_last_rebalance_ts(&e));
    assert_eq!(last_rebalance_before, 0);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // First rebalance should succeed without time threshold check
    client.rebalance(&admin, &RebalanceParams { target_nav: None });

    // Verify timestamp updated
    let last_rebalance_after = e.as_contract(&contract_address, || get_last_rebalance_ts(&e));
    assert!(last_rebalance_after > 0);
}

// ===== Event Verification =====

// Note: Event verification would require checking emitted events
// Soroban SDK provides e.events() for this, but we'll keep tests focused on state changes
// and leave detailed event testing for integration tests

#[test]
fn test_rebalance_executed_event() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token, 10000)]);

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Execute rebalance - events will be emitted
    client.rebalance(&admin, &RebalanceParams { target_nav: None });

    // In a full implementation, we would:
    // - Get events with e.events().all()
    // - Filter for "rebalance_executed" event
    // - Verify it includes nav_before, nav_after, components_before, components_after
}

#[test]
fn test_rebalance_completed_detailed_event() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexFundClient::new(&e, &contract_address);

    let token = create_mock_token(&e);
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 10000)]);

    // Set up imbalanced state to generate swaps
    e.as_contract(&contract_address, || {
        set_base_nav(&e, &100_000);
        set_component_balance(&e, token, 50_000); // Different from target
    });

    // Allow immediate rebalance
    allow_immediate_rebalance(&e, &contract_address);

    // Execute rebalance with swaps
    client.rebalance(&admin, &RebalanceParams { target_nav: None });

    // Events should include: total_swaps, performance_delta, duration_ms
}
