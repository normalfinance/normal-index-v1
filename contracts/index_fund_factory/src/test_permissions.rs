#![cfg(test)]

use crate::testutils::Setup;
use access_control::constants::ADMIN_ACTIONS_DELAY;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Symbol};
use utils::test_utils::{install_dummy_wasm, jump};

// test admin transfer ownership
#[test]
#[should_panic(expected = "Error(Contract, #2908)")]
fn test_admin_transfer_ownership_too_early() {
    let setup = Setup::default();
    let factory = setup.factory;
    let admin_original = setup.users[0].clone();
    let admin_new = Address::generate(&setup.env);

    factory.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(factory
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY - 1);
    factory.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2906)")]
fn test_admin_transfer_ownership_twice() {
    let setup = Setup::default();
    let factory = setup.factory;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    factory.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    factory.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_admin_transfer_ownership_not_committed() {
    let setup = Setup::default();
    let factory = setup.factory;
    let admin_original = setup.admin;

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    factory.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_admin_transfer_ownership_reverted() {
    let setup = Setup::default();
    let factory = setup.factory;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    factory.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(factory
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    factory.revert_transfer_ownership(&admin_original, &symbol_short!("Admin"));
    factory.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
fn test_admin_transfer_ownership() {
    let setup = Setup::default();
    let factory = setup.factory;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    factory.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(factory
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    factory.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));

    factory.commit_transfer_ownership(&admin_new, &symbol_short!("Admin"), &admin_new);
}

// test emergency admin transfer ownership
#[test]
#[should_panic(expected = "Error(Contract, #2908)")]
fn test_emergency_admin_transfer_ownership_too_early() {
    let setup = Setup::default();
    let factory = setup.factory;
    let emergency_admin_new = Address::generate(&setup.env);

    factory.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(factory
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(factory
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY - 1);
    factory.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2906)")]
fn test_emergency_admin_transfer_ownership_twice() {
    let setup = Setup::default();
    let factory = setup.factory;
    let emergency_admin_new = Address::generate(&setup.env);

    factory.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
    factory.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_emergency_admin_transfer_ownership_not_committed() {
    let setup = Setup::default();
    let factory = setup.factory;

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    factory.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_emergency_admin_transfer_ownership_reverted() {
    let setup = Setup::default();
    let factory = setup.factory;
    let emergency_admin_new = Address::generate(&setup.env);

    factory.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(factory
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(factory
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    factory.revert_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
    factory.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
fn test_emergency_admin_transfer_ownership() {
    let setup = Setup::default();
    let factory = setup.factory;
    let emergency_admin_new = Address::generate(&setup.env);

    factory.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(factory
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(factory
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    factory.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    // check emergency admin has changed
    assert!(factory
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_ok());
    assert!(factory
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_err());
}

#[test]
fn test_transfer_ownership_separate_deadlines() {
    let setup = Setup::default();
    let factory = setup.factory;
    let admin_new = Address::generate(&setup.env);
    let emergency_admin_new = Address::generate(&setup.env);

    assert_eq!(
        factory.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        setup.emergency_admin
    );
    assert_eq!(
        factory.get_future_address(&symbol_short!("Admin")),
        setup.admin
    );

    assert!(factory
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(factory
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    factory.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
    jump(&setup.env, 10);
    factory.commit_transfer_ownership(&setup.admin, &symbol_short!("Admin"), &admin_new);

    assert_eq!(
        factory.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        emergency_admin_new
    );
    assert_eq!(
        factory.get_future_address(&symbol_short!("Admin")),
        admin_new
    );

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1 - 10);
    factory.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
    assert!(factory
        .try_apply_transfer_ownership(&setup.admin, &symbol_short!("Admin"))
        .is_err());

    assert_eq!(
        factory.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        emergency_admin_new
    );

    jump(&setup.env, 10);
    factory.apply_transfer_ownership(&setup.admin, &symbol_short!("Admin"));

    assert_eq!(
        factory.get_future_address(&symbol_short!("Admin")),
        admin_new
    );

    // check ownership transfer is complete. new admin is capable to call protected methods
    //      and new emergency admin can change toggle emergency mode
    factory.commit_transfer_ownership(&admin_new, &Symbol::new(&setup.env, "Admin"), &setup.admin);
    assert!(factory
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_ok());
    assert!(factory
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_err());
}

// upgrade factory
#[test]
fn test_commit_upgrade() {
    let setup = Setup::default();
    let factory = setup.factory;
    let new_wasm = install_dummy_wasm(&setup.env);
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin, true),
        (setup.emergency_admin, false),
        (setup.rewards_admin, false),
        (setup.operations_admin, false),
        (setup.fee_admin, false),
    ] {
        assert_eq!(factory.try_commit_upgrade(&addr, &new_wasm).is_ok(), is_ok);
    }
}

#[test]
fn test_apply_upgrade_third_party_user() {
    let setup = Setup::default();
    let factory = setup.factory;
    let user = Address::generate(&setup.env);
    factory.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(factory.try_apply_upgrade(&user).is_err());
}

#[test]
fn test_apply_upgrade_emergency_admin() {
    let setup = Setup::default();
    let factory = setup.factory;
    factory.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(factory.try_apply_upgrade(&setup.emergency_admin).is_err());
}

#[test]
fn test_apply_upgrade_admin() {
    let setup = Setup::default();
    let factory = setup.factory;
    let new_wasm = install_dummy_wasm(&setup.env);

    assert_ne!(factory.version(), 130);

    factory.commit_upgrade(&setup.admin, &new_wasm);
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert_eq!(factory.apply_upgrade(&setup.admin), new_wasm);

    // check contracts updated, dummy contract version is 130
    assert_eq!(factory.version(), 130);
}

// emergency mode
#[test]
fn test_set_emergency_mode_third_party_user() {
    let setup = Setup::default();
    let factory = setup.factory;
    let user = Address::generate(&setup.env);
    assert!(factory.try_set_emergency_mode(&user, &false).is_err());
}

#[test]
fn test_set_emergency_mode_admin() {
    let setup = Setup::default();
    let factory = setup.factory;
    assert!(factory
        .try_set_emergency_mode(&setup.admin, &false)
        .is_err());
}

#[test]
fn test_set_emergency_mode_emergency_admin() {
    let setup = Setup::default();
    let factory = setup.factory;
    assert!(factory
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());
}

// manage privileged addresses
#[test]
fn test_set_privileged_addresses() {
    let setup = Setup::default();
    let factory = setup.factory;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin.clone(), true),
        (setup.rewards_admin.clone(), false),
        (setup.operations_admin.clone(), false),
        (setup.fee_admin.clone(), false),
    ] {
        assert_eq!(
            factory
                .try_set_privileged_addrs(
                    &addr,
                    &setup.rewards_admin,
                    &setup.operations_admin,
                    &setup.fee_admin
                )
                .is_ok(),
            is_ok
        );
    }
}

#[test]
fn test_update_index_contract_wasm() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let new_wasm = install_dummy_wasm(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin, true),
        (setup.rewards_admin, false),
        (setup.operations_admin, true),
        (setup.fee_admin, false),
    ] {
        assert_eq!(
            setup
                .factory
                .try_set_index_contract_wasm(&addr, &new_wasm)
                .is_ok(),
            is_ok
        );
    }
}

#[test]
fn test_update_index_token_wasm() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let new_wasm = install_dummy_wasm(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin, true),
        (setup.rewards_admin, false),
        (setup.operations_admin, true),
        (setup.fee_admin, false),
    ] {
        assert_eq!(
            setup
                .factory
                .try_set_index_token_wasm(&addr, &new_wasm)
                .is_ok(),
            is_ok
        );
    }
}

#[test]
fn test_update_adapter_registry() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let registry = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin, true),
        (setup.rewards_admin, false),
        (setup.operations_admin, true),
        (setup.fee_admin, false),
    ] {
        assert_eq!(
            setup
                .factory
                .try_set_adapter_registry(&addr, &registry)
                .is_ok(),
            is_ok
        );
    }
}
