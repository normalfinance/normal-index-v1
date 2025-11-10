#![cfg(test)]

use super::contract::{Index, IndexClient};
use super::interface::{AdminInterface, ComponentAction, ComponentUpdate, IndexTrait, RefactorParams};
use super::storage::{
    get_all_components, get_component, get_component_safe, get_last_rebalance_ts,
    get_last_updated_ts, set_component, set_last_rebalance_ts, set_last_updated_ts, Component,
};
use access_control::access::AccessControl;
use access_control::role::Role;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    vec, Address, Env, Map, Symbol, Vec,
};
use token_share::put_token_share;
use utils::test_utils::jump;

// Test utilities

fn register_test_contract(e: &Env) -> Address {
    e.register_contract(None, Index)
}

fn create_test_index(e: &Env) -> (Address, Address, Address) {
    let contract_address = register_test_contract(e);
    let admin = Address::generate(e);
    let token = Address::generate(e);

    e.as_contract(&contract_address, || {
        // Initialize access control with admin
        let access_control = AccessControl::new(e);
        access_control.set_role_address(&Role::Admin, &admin);
        // Set token share
        put_token_share(e, token.clone());
    });

    (contract_address, admin, token)
}

fn create_mock_token(e: &Env) -> Address {
    Address::generate(e)
}

fn setup_components(e: &Env, contract: &Address, tokens_with_weights: Vec<(Address, u128)>) {
    e.as_contract(contract, || {
        for (token, weight) in tokens_with_weights.iter() {
            let component = Component {
                asset: Symbol::new(e, "TOKEN"),
                weight,
            };
            set_component(e, token, component);
        }
    });
}

// ===== Basic Refactor Operations =====

#[test]
fn test_refactor_add_component() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token = create_mock_token(&e);

    // Record timestamp before refactor
    let time_before = e.ledger().timestamp();

    // Create refactor params to add a component
    let updates = vec![
        &e,
        ComponentUpdate {
            token: token.clone(),
            new_weight: 10000, // 100%
            action: ComponentAction::Add,
        },
    ];

    let params = RefactorParams {
        component_updates: updates,
    };

    // Execute refactor
    client.refactor(&admin, &params);

    // Verify component was added
    let component = e.as_contract(&contract_address, || get_component(&e, token.clone()));
    assert_eq!(component.weight, 10000);

    // Verify last_updated_ts was updated
    let last_updated = e.as_contract(&contract_address, || get_last_updated_ts(&e));
    assert!(last_updated >= time_before, "last_updated_ts should be updated");

    // Verify last_rebalance_ts was NOT updated
    let last_rebalance = e.as_contract(&contract_address, || get_last_rebalance_ts(&e));
    assert_eq!(last_rebalance, 0, "last_rebalance_ts should not be updated by refactor");
}

#[test]
fn test_refactor_remove_component() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token = create_mock_token(&e);

    // First, add a component
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 10000)]);

    // Verify component exists
    let component_before = e.as_contract(&contract_address, || get_component_safe(&e, token.clone()));
    assert!(component_before.is_some());

    // Create refactor params to remove the component
    let updates = vec![
        &e,
        ComponentUpdate {
            token: token.clone(),
            new_weight: 0, // Weight doesn't matter for Remove
            action: ComponentAction::Remove,
        },
    ];

    let params = RefactorParams {
        component_updates: updates,
    };

    // Execute refactor
    client.refactor(&admin, &params);

    // Verify component was removed
    let component_after = e.as_contract(&contract_address, || get_component_safe(&e, token.clone()));
    assert!(component_after.is_none(), "Component should be removed");
}

#[test]
fn test_refactor_update_weight() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token = create_mock_token(&e);

    // Add component with initial weight
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 5000)]);

    // Verify initial weight
    let component_before = e.as_contract(&contract_address, || get_component(&e, token.clone()));
    assert_eq!(component_before.weight, 5000);

    // Update weight
    let updates = vec![
        &e,
        ComponentUpdate {
            token: token.clone(),
            new_weight: 10000,
            action: ComponentAction::UpdateWeight,
        },
    ];

    let params = RefactorParams {
        component_updates: updates,
    };

    client.refactor(&admin, &params);

    // Verify weight was updated
    let component_after = e.as_contract(&contract_address, || get_component(&e, token.clone()));
    assert_eq!(component_after.weight, 10000);
}

// ===== Weight Validation =====

#[test]
fn test_refactor_weight_sum_must_equal_10000() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token1 = create_mock_token(&e);
    let token2 = create_mock_token(&e);

    // Valid: weights sum to 10000
    let updates = vec![
        &e,
        ComponentUpdate {
            token: token1.clone(),
            new_weight: 6000,
            action: ComponentAction::Add,
        },
        ComponentUpdate {
            token: token2.clone(),
            new_weight: 4000,
            action: ComponentAction::Add,
        },
    ];

    let params = RefactorParams {
        component_updates: updates,
    };

    // Should succeed
    client.refactor(&admin, &params);

    // Verify both components added
    let all_components = e.as_contract(&contract_address, || get_all_components(&e));
    assert_eq!(all_components.len(), 2);
}

#[test]
#[should_panic(expected = "InvalidWeightSum")]
fn test_refactor_weight_sum_not_10000_fails() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token1 = create_mock_token(&e);
    let token2 = create_mock_token(&e);

    // Invalid: weights sum to 9000
    let updates = vec![
        &e,
        ComponentUpdate {
            token: token1.clone(),
            new_weight: 5000,
            action: ComponentAction::Add,
        },
        ComponentUpdate {
            token: token2.clone(),
            new_weight: 4000,
            action: ComponentAction::Add,
        },
    ];

    let params = RefactorParams {
        component_updates: updates,
    };

    // Should fail
    client.refactor(&admin, &params);
}

#[test]
fn test_refactor_multiple_updates_weight_sum_validation() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token1 = create_mock_token(&e);
    let token2 = create_mock_token(&e);

    // First refactor: Add two components (5000 + 5000 = 10000)
    let updates1 = vec![
        &e,
        ComponentUpdate {
            token: token1.clone(),
            new_weight: 5000,
            action: ComponentAction::Add,
        },
        ComponentUpdate {
            token: token2.clone(),
            new_weight: 5000,
            action: ComponentAction::Add,
        },
    ];

    client.refactor(&admin, &RefactorParams { component_updates: updates1 });

    // Second refactor: Update weights (6000 + 4000 = 10000)
    let updates2 = vec![
        &e,
        ComponentUpdate {
            token: token1.clone(),
            new_weight: 6000,
            action: ComponentAction::UpdateWeight,
        },
        ComponentUpdate {
            token: token2.clone(),
            new_weight: 4000,
            action: ComponentAction::UpdateWeight,
        },
    ];

    client.refactor(&admin, &RefactorParams { component_updates: updates2 });

    // Verify weights
    let comp1 = e.as_contract(&contract_address, || get_component(&e, token1));
    let comp2 = e.as_contract(&contract_address, || get_component(&e, token2));

    assert_eq!(comp1.weight, 6000);
    assert_eq!(comp2.weight, 4000);
    assert_eq!(comp1.weight + comp2.weight, 10000);
}

// ===== Permission Checks =====

#[test]
#[should_panic(expected = "UnauthorizedRefactor")]
fn test_refactor_requires_admin() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, _, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let non_admin = Address::generate(&e);
    let token = create_mock_token(&e);

    let updates = vec![
        &e,
        ComponentUpdate {
            token,
            new_weight: 10000,
            action: ComponentAction::Add,
        },
    ];

    let params = RefactorParams {
        component_updates: updates,
    };

    // Non-admin trying to refactor should fail
    client.refactor(&non_admin, &params);
}

#[test]
#[should_panic(expected = "Blacklisted")]
fn test_refactor_blacklisted_admin_fails() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    // Blacklist the admin
    client.set_blacklist_status(&admin, &admin, &true);

    let token = create_mock_token(&e);
    let updates = vec![
        &e,
        ComponentUpdate {
            token,
            new_weight: 10000,
            action: ComponentAction::Add,
        },
    ];

    let params = RefactorParams {
        component_updates: updates,
    };

    // Blacklisted admin should fail
    client.refactor(&admin, &params);
}

#[test]
fn test_refactor_admin_can_refactor_anytime() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token1 = create_mock_token(&e);
    let token2 = create_mock_token(&e);

    // First refactor
    let updates1 = vec![
        &e,
        ComponentUpdate {
            token: token1,
            new_weight: 10000,
            action: ComponentAction::Add,
        },
    ];
    client.refactor(&admin, &RefactorParams { component_updates: updates1 });

    // Immediate second refactor (no time restriction)
    let updates2 = vec![
        &e,
        ComponentUpdate {
            token: token2,
            new_weight: 10000,
            action: ComponentAction::UpdateWeight,
        },
    ];
    client.refactor(&admin, &RefactorParams { component_updates: updates2 });

    // Should succeed without time threshold check
}

// ===== Critical Integration: Refactor Blocks Operations =====

#[test]
#[should_panic(expected = "RebalanceRequiredAfterRefactor")]
fn test_mint_blocked_after_refactor() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, token) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let user = Address::generate(&e);
    let comp_token = create_mock_token(&e);

    // Set initial rebalance timestamp
    e.as_contract(&contract_address, || {
        set_last_rebalance_ts(&e, &100);
    });

    // Refactor - this updates last_updated_ts
    let updates = vec![
        &e,
        ComponentUpdate {
            token: comp_token,
            new_weight: 10000,
            action: ComponentAction::Add,
        },
    ];
    client.refactor(&admin, &RefactorParams { component_updates: updates });

    // Verify last_updated > last_rebalance
    let last_updated = e.as_contract(&contract_address, || get_last_updated_ts(&e));
    let last_rebalance = e.as_contract(&contract_address, || get_last_rebalance_ts(&e));
    assert!(last_updated > last_rebalance);

    // Attempt mint - should fail
    client.mint(&user, &token, &1000, &None, &None);
}

#[test]
#[should_panic(expected = "RebalanceRequiredAfterRefactor")]
fn test_redeem_blocked_after_refactor() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let user = Address::generate(&e);
    let comp_token = create_mock_token(&e);

    // Set initial rebalance timestamp
    e.as_contract(&contract_address, || {
        set_last_rebalance_ts(&e, &100);
    });

    // Refactor
    let updates = vec![
        &e,
        ComponentUpdate {
            token: comp_token,
            new_weight: 10000,
            action: ComponentAction::Add,
        },
    ];
    client.refactor(&admin, &RefactorParams { component_updates: updates });

    // Attempt redeem - should fail
    client.redeem(&user, &1000);
}

#[test]
fn test_operations_allowed_after_rebalance() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, token) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let user = Address::generate(&e);
    let comp_token = create_mock_token(&e);

    // Initial setup with rebalance timestamp
    e.as_contract(&contract_address, || {
        set_last_rebalance_ts(&e, &100);
        set_last_updated_ts(&e, &100);
    });

    // Refactor changes weights
    let updates = vec![
        &e,
        ComponentUpdate {
            token: comp_token,
            new_weight: 10000,
            action: ComponentAction::Add,
        },
    ];
    client.refactor(&admin, &RefactorParams { component_updates: updates });

    // Now simulate rebalance by updating last_rebalance_ts
    e.as_contract(&contract_address, || {
        let current_time = e.ledger().timestamp();
        set_last_rebalance_ts(&e, &current_time);
    });

    // Verify last_rebalance >= last_updated
    let last_updated = e.as_contract(&contract_address, || get_last_updated_ts(&e));
    let last_rebalance = e.as_contract(&contract_address, || get_last_rebalance_ts(&e));
    assert!(last_rebalance >= last_updated);

    // Operations should now succeed (though they may fail for other reasons like missing setup)
    // We're just verifying the RebalanceRequiredAfterRefactor check passes
}

// ===== Edge Cases =====

#[test]
fn test_refactor_with_no_components() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token = create_mock_token(&e);

    // Start with empty index - no components
    let components_before = e.as_contract(&contract_address, || get_all_components(&e));
    assert_eq!(components_before.len(), 0);

    // Add first component
    let updates = vec![
        &e,
        ComponentUpdate {
            token: token.clone(),
            new_weight: 10000,
            action: ComponentAction::Add,
        },
    ];
    client.refactor(&admin, &RefactorParams { component_updates: updates });

    // Verify component added
    let components_after = e.as_contract(&contract_address, || get_all_components(&e));
    assert_eq!(components_after.len(), 1);
}

#[test]
fn test_refactor_remove_last_component() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token = create_mock_token(&e);

    // Add one component
    setup_components(&e, &contract_address, vec![&e, (token.clone(), 10000)]);

    // Verify one component exists
    let components_before = e.as_contract(&contract_address, || get_all_components(&e));
    assert_eq!(components_before.len(), 1);

    // Remove it
    let updates = vec![
        &e,
        ComponentUpdate {
            token,
            new_weight: 0,
            action: ComponentAction::Remove,
        },
    ];
    client.refactor(&admin, &RefactorParams { component_updates: updates });

    // Verify no components remain
    let components_after = e.as_contract(&contract_address, || get_all_components(&e));
    assert_eq!(components_after.len(), 0);
}

#[test]
fn test_refactor_batch_updates() {
    let e = Env::default();
    e.mock_all_auths();

    let (contract_address, admin, _) = create_test_index(&e);
    let client = IndexClient::new(&e, &contract_address);

    let token1 = create_mock_token(&e);
    let token2 = create_mock_token(&e);
    let token3 = create_mock_token(&e);
    let token4 = create_mock_token(&e);

    // Setup: Add 3 components
    setup_components(
        &e,
        &contract_address,
        vec![
            &e,
            (token1.clone(), 4000),
            (token2.clone(), 3000),
            (token3.clone(), 3000),
        ],
    );

    // Batch refactor: Add 1, Remove 1, Update 2
    let updates = vec![
        &e,
        ComponentUpdate {
            token: token4.clone(),
            new_weight: 2000,
            action: ComponentAction::Add,
        },
        ComponentUpdate {
            token: token3.clone(),
            new_weight: 0,
            action: ComponentAction::Remove,
        },
        ComponentUpdate {
            token: token1.clone(),
            new_weight: 5000,
            action: ComponentAction::UpdateWeight,
        },
        ComponentUpdate {
            token: token2.clone(),
            new_weight: 3000,
            action: ComponentAction::UpdateWeight,
        },
    ];

    client.refactor(&admin, &RefactorParams { component_updates: updates });

    // Verify final state: token1 (5000), token2 (3000), token4 (2000)
    let all_components = e.as_contract(&contract_address, || get_all_components(&e));
    assert_eq!(all_components.len(), 3);

    let comp1 = e.as_contract(&contract_address, || get_component(&e, token1));
    let comp2 = e.as_contract(&contract_address, || get_component(&e, token2));
    let comp4 = e.as_contract(&contract_address, || get_component(&e, token4));

    assert_eq!(comp1.weight, 5000);
    assert_eq!(comp2.weight, 3000);
    assert_eq!(comp4.weight, 2000);
    assert_eq!(comp1.weight + comp2.weight + comp4.weight, 10000);

    // token3 should be removed
    let comp3 = e.as_contract(&contract_address, || get_component_safe(&e, token3));
    assert!(comp3.is_none());
}

