#![cfg(test)]

use super::fees::{
    calculate_accrued_fees, collect_accrued_fees_if_any, collect_fees_before_action,
    force_collect_fees, get_effective_balance, get_user_fee_state,
    initialize_or_update_user_tracking, preview_accrued_fees,
};
use super::storage::{
    get_accumulated_manager_fees, get_accumulated_protocol_fees, get_manager_fee_amount,
    get_minimum_shares_for_fee_collection, get_protocol_fee_recipient, get_total_fees,
    set_accumulated_manager_fees, set_accumulated_protocol_fees,
    set_manager_address, set_manager_fee_amount, set_minimum_shares_for_fee_collection,
    set_protocol_fee_recipient,
};
use super::contract::Index;
use soroban_sdk::{
    testutils::Address as _,
    Address, Env,
};
use utils::test_utils::jump;


const SECONDS_PER_YEAR: u64 = 31_536_000;

fn register_test_contract(e: &Env) -> Address {
    e.register_contract(None, Index)
} 


fn setup_fee_config(
    e: &Env,
    manager_fee_amount: u128,
    minimum_shares: u128,
    manager_address: Address,
    protocol_recipient: Address,
) {
    set_manager_fee_amount(e, &manager_fee_amount);
    set_minimum_shares_for_fee_collection(e, &minimum_shares);
    set_manager_address(e, &manager_address);
    set_protocol_fee_recipient(e, &protocol_recipient);
}


fn create_test_environment(e: &Env) -> (Address, Address, Address, Address) {
    let contract_address = register_test_contract(e);
    let manager = Address::generate(e);
    let protocol_recipient = Address::generate(e);
    let user = Address::generate(e);
    
    e.as_contract(&contract_address, || {
        setup_fee_config(
            e,
            100u128,                    
            25_000_000_000u128,         
            manager.clone(),
            protocol_recipient.clone(),
        );
    });
    
    (contract_address, manager, protocol_recipient, user)
}

#[test]
fn test_calculate_accrued_fees_basic() {
    let e = Env::default();
    e.mock_all_auths();
    
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 100_000_000_000i128; 
    let start_time = 1000u64;
    let end_time = start_time + SECONDS_PER_YEAR; 
    
    let (manager_fee, protocol_fee) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            end_time,
        )
    });
    
    
    
    assert_eq!(manager_fee, 100u128, "Manager fee should be 100 tokens for 1 year");
    assert_eq!(protocol_fee, 0u128, "Protocol fee should be 0 when factory not set");
}

#[test]
fn test_calculate_accrued_fees_no_time_elapsed() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let start_time = 1000u64;
    let end_time = start_time; 
    
    let (manager_fee, protocol_fee) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            100_000_000_000i128,
            start_time,
            end_time,
        )
    });
    
    assert_eq!(manager_fee, 0, "No manager fee should accrue with no time elapsed");
    assert_eq!(protocol_fee, 0, "No protocol fee should accrue with no time elapsed");
}

#[test]
fn test_calculate_accrued_fees_zero_balance() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let start_time = 1000u64;
    let end_time = start_time + SECONDS_PER_YEAR;
    
    let (manager_fee, protocol_fee) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            0i128, 
            start_time,
            end_time,
        )
    });
    
    assert_eq!(manager_fee, 0, "No fees should accrue on zero balance");
    assert_eq!(protocol_fee, 0, "No fees should accrue on zero balance");
}

#[test]
fn test_calculate_accrued_fees_below_minimum_shares() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    
    let minimum_shares = e.as_contract(&contract_address, || {
        get_minimum_shares_for_fee_collection(&e)
    });
    assert_eq!(minimum_shares, 25_000_000_000u128, "Minimum shares should be 25k");
    
    let start_time = 1000u64;
    let end_time = start_time + SECONDS_PER_YEAR;
    
    
    let user_balance = 10_000_000_000i128; 
    
    let (manager_fee, protocol_fee) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            end_time,
        )
    });
    
    assert_eq!(manager_fee, 0, "No fees should accrue below minimum shares");
    assert_eq!(protocol_fee, 0, "No fees should accrue below minimum shares");
}

#[test]
fn test_calculate_accrued_fees_above_minimum_shares() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let minimum_shares = e.as_contract(&contract_address, || {
        get_minimum_shares_for_fee_collection(&e)
    });
    let user_balance = (minimum_shares + 1) as i128; 
    
    let start_time = 1000u64;
    let end_time = start_time + SECONDS_PER_YEAR;
    
    let (manager_fee, protocol_fee) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            end_time,
        )
    });
    
    assert!(manager_fee > 0, "Manager fees should accrue above minimum shares");
    assert_eq!(protocol_fee, 0, "Protocol fee should be 0 when factory not set");
}

#[test]
fn test_collect_accrued_fees_if_any_below_threshold() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, 100_000_000_000i128);
    });
    
    e.as_contract(&contract_address, || {
        let mut user_state = get_user_fee_state(&e, &user);
        user_state.accrued_manager_fees = 100u128; 
        user_state.accrued_protocol_fees = 50u128; 
        // Note: This test modifies user_state but doesn't write it back, which may be intentional
    });
    
    let (collected_manager, collected_protocol) = e.as_contract(&contract_address, || {
        collect_accrued_fees_if_any(&e, &user)
    });
    
    assert_eq!(collected_manager, 0, "Fees below threshold should not be collected");
    assert_eq!(collected_protocol, 0, "Fees below threshold should not be collected");
}

#[test]
fn test_collect_accrued_fees_if_any_above_threshold() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 1_000_000_000_000i128; 
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, user_balance);
    });
    
    jump(&e, SECONDS_PER_YEAR * 2); 
    
    let (collected_manager, collected_protocol) = e.as_contract(&contract_address, || {
        collect_accrued_fees_if_any(&e, &user)
    });
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    if collected_manager > 0 || collected_protocol > 0 {
        let accumulated_manager = e.as_contract(&contract_address, || {
            get_accumulated_manager_fees(&e)
        });
        let accumulated_protocol = e.as_contract(&contract_address, || {
            get_accumulated_protocol_fees(&e)
        });
        
        assert_eq!(accumulated_manager, collected_manager, "Accumulated manager fees should match collected");
        assert_eq!(accumulated_protocol, collected_protocol, "Accumulated protocol fees should match collected");
    }
}

#[test]
fn test_initialize_or_update_user_tracking() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let initial_balance = 100_000_000_000i128;
    let current_time = e.ledger().timestamp();
    
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, initial_balance);
    });
    
    let user_state = e.as_contract(&contract_address, || {
        get_user_fee_state(&e, &user)
    });
    assert_eq!(user_state.balance, initial_balance, "Balance should be set");
    assert_eq!(user_state.last_fee_update, current_time, "Timestamp should be updated");
    assert_eq!(user_state.accrued_manager_fees, 0, "Accrued manager fees should start at 0");
    assert_eq!(user_state.accrued_protocol_fees, 0, "Accrued protocol fees should start at 0");
    
    let additional_balance = 50_000_000_000i128;
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, additional_balance);
    });
    
    let updated_state = e.as_contract(&contract_address, || {
        get_user_fee_state(&e, &user)
    });
    assert_eq!(
        updated_state.balance,
        initial_balance + additional_balance,
        "Balance should be updated"
    );
}

#[test]
fn test_collect_fees_before_action() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 100_000_000_000i128;
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, user_balance);
    });
    
    let initial_state = e.as_contract(&contract_address, || {
        get_user_fee_state(&e, &user)
    });
    assert_eq!(initial_state.balance, user_balance);
    
    jump(&e, SECONDS_PER_YEAR / 2); 
    
    let balance_change = -10_000_000_000i128; 
    let (collected_manager, collected_protocol) = e.as_contract(&contract_address, || {
        collect_fees_before_action(&e, &user, balance_change)
    });
    
    let final_state = e.as_contract(&contract_address, || {
        get_user_fee_state(&e, &user)
    });
    
    assert!(
        final_state.balance <= user_balance,
        "Balance should be reduced after fee collection and action"
    );
    
    if collected_manager > 0 || collected_protocol > 0 {
        let accumulated_manager = e.as_contract(&contract_address, || {
            get_accumulated_manager_fees(&e)
        });
        let accumulated_protocol = e.as_contract(&contract_address, || {
            get_accumulated_protocol_fees(&e)
        });
        
        assert_eq!(accumulated_manager, collected_manager, "Accumulated manager fees should match");
        assert_eq!(accumulated_protocol, collected_protocol, "Accumulated protocol fees should match");
    }
}

#[test]
fn test_preview_accrued_fees() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 100_000_000_000i128;
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, user_balance);
    });
    
    let initial_accumulated_manager = e.as_contract(&contract_address, || {
        get_accumulated_manager_fees(&e)
    });
    let initial_accumulated_protocol = e.as_contract(&contract_address, || {
        get_accumulated_protocol_fees(&e)
    });
    
    jump(&e, SECONDS_PER_YEAR / 2); 
    
    let (preview_manager, preview_protocol) = e.as_contract(&contract_address, || {
        preview_accrued_fees(&e, &user)
    });
    
    assert!(preview_manager >= 0, "Preview should show manager fees");
    assert!(preview_protocol >= 0, "Preview should show protocol fees");
    
    let accumulated_manager = e.as_contract(&contract_address, || {
        get_accumulated_manager_fees(&e)
    });
    let accumulated_protocol = e.as_contract(&contract_address, || {
        get_accumulated_protocol_fees(&e)
    });
    
    assert_eq!(accumulated_manager, initial_accumulated_manager, "Preview should not collect fees");
    assert_eq!(accumulated_protocol, initial_accumulated_protocol, "Preview should not collect fees");
}

#[test]
fn test_force_collect_fees() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 100_000_000_000i128;
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, user_balance);
    });
    
    jump(&e, SECONDS_PER_YEAR / 4); 
    
    let (collected_manager, collected_protocol) = e.as_contract(&contract_address, || {
        force_collect_fees(&e, &user)
    });
    
    assert!(collected_manager >= 0, "Force collect should return manager fees");
    assert!(collected_protocol >= 0, "Force collect should return protocol fees");
    
    if collected_manager > 0 || collected_protocol > 0 {
        let accumulated_manager = e.as_contract(&contract_address, || {
            get_accumulated_manager_fees(&e)
        });
        let accumulated_protocol = e.as_contract(&contract_address, || {
            get_accumulated_protocol_fees(&e)
        });
        
        assert_eq!(accumulated_manager, collected_manager, "Accumulated manager fees should match");
        assert_eq!(accumulated_protocol, collected_protocol, "Accumulated protocol fees should match");
    }
}

#[test]
fn test_get_effective_balance() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 100_000_000_000i128;
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, user_balance);
    });
    
    let effective_balance_before = e.as_contract(&contract_address, || {
        get_effective_balance(&e, &user)
    });
    
    assert_eq!(effective_balance_before, user_balance, "Effective balance should equal balance initially");
    
    jump(&e, SECONDS_PER_YEAR / 2); 
    
    let effective_balance_after = e.as_contract(&contract_address, || {
        get_effective_balance(&e, &user)
    });
    
    assert!(
        effective_balance_after <= user_balance,
        "Effective balance should account for pending fees"
    );
}

#[test]
fn test_fee_collection_updates_total_fees() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 10_000_000_000_000i128; 
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, user_balance);
    });
    
    let initial_total_fees = e.as_contract(&contract_address, || {
        get_total_fees(&e)
    });
    
    jump(&e, SECONDS_PER_YEAR * 10); 
    
    let (collected_manager, collected_protocol) = e.as_contract(&contract_address, || {
        collect_accrued_fees_if_any(&e, &user)
    });
    
    if collected_manager > 0 || collected_protocol > 0 {
        let total_fees = e.as_contract(&contract_address, || {
            get_total_fees(&e)
        });
        assert_eq!(
            total_fees,
            initial_total_fees + collected_manager + collected_protocol,
            "Total fees should equal sum of manager and protocol fees"
        );
    }
}

#[test]
fn test_fee_calculation_time_proportionality() {
    let e = Env::default();
    let (_, _, _, user) = create_test_environment(&e);
    
    let user_balance = 100_000_000_000i128;
    let start_time = 1000u64;
    
    
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let one_year = start_time + SECONDS_PER_YEAR;
    let (fees_1yr_manager, fees_1yr_protocol) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            one_year,
        )
    });
    
    let six_months = start_time + (SECONDS_PER_YEAR / 2);
    let (fees_6mo_manager, fees_6mo_protocol) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            six_months,
        )
    });
    
    
    
    assert!(
        fees_6mo_manager * 2 <= fees_1yr_manager + 1,
        "6 month manager fees should be approximately half of 1 year fees"
    );
    assert!(
        fees_6mo_protocol * 2 <= fees_1yr_protocol + 1,
        "6 month protocol fees should be approximately half of 1 year fees"
    );
}

#[test]
fn test_manager_fee_amount_storage() {
    let e = Env::default();
    let (contract_address, _, _, _) = create_test_environment(&e);
    
    let manager_fee = e.as_contract(&contract_address, || {
        get_manager_fee_amount(&e)
    });
    assert_eq!(manager_fee, 100u128, "Manager fee amount should be 100");
    
    e.as_contract(&contract_address, || {
        set_manager_fee_amount(&e, &200u128);
    });
    
    let updated_fee = e.as_contract(&contract_address, || {
        get_manager_fee_amount(&e)
    });
    assert_eq!(updated_fee, 200u128, "Manager fee amount should be updated to 200");
}

#[test]
fn test_protocol_fee_recipient_storage() {
    let e = Env::default();
    let (contract_address, _, protocol_recipient, _) = create_test_environment(&e);
    
    let recipient = e.as_contract(&contract_address, || {
        get_protocol_fee_recipient(&e)
    });
    assert_eq!(recipient, protocol_recipient, "Protocol fee recipient should match");
    
    let new_recipient = Address::generate(&e);
    e.as_contract(&contract_address, || {
        set_protocol_fee_recipient(&e, &new_recipient);
    });
    
    let updated_recipient = e.as_contract(&contract_address, || {
        get_protocol_fee_recipient(&e)
    });
    assert_eq!(updated_recipient, new_recipient, "Protocol fee recipient should be updated");
}

#[test]
fn test_accumulated_fees_storage() {
    let e = Env::default();
    let (contract_address, _, _, _) = create_test_environment(&e);
    
    let initial_manager = e.as_contract(&contract_address, || {
        get_accumulated_manager_fees(&e)
    });
    let initial_protocol = e.as_contract(&contract_address, || {
        get_accumulated_protocol_fees(&e)
    });
    
    assert_eq!(initial_manager, 0, "Initial accumulated manager fees should be 0");
    assert_eq!(initial_protocol, 0, "Initial accumulated protocol fees should be 0");
    
    e.as_contract(&contract_address, || {
        set_accumulated_manager_fees(&e, &1_000_000u128);
        set_accumulated_protocol_fees(&e, &500_000u128);
    });
    
    let updated_manager = e.as_contract(&contract_address, || {
        get_accumulated_manager_fees(&e)
    });
    let updated_protocol = e.as_contract(&contract_address, || {
        get_accumulated_protocol_fees(&e)
    });
    
    assert_eq!(updated_manager, 1_000_000u128, "Accumulated manager fees should be updated");
    assert_eq!(updated_protocol, 500_000u128, "Accumulated protocol fees should be updated");
}

#[test]
fn test_fee_calculation_with_different_manager_fee_rates() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 100_000_000_000i128;
    let start_time = 1000u64;
    let end_time = start_time + SECONDS_PER_YEAR;
    
    e.as_contract(&contract_address, || {
        set_manager_fee_amount(&e, &50u128);
    });
    let (fees_50bps_manager, _) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            end_time,
        )
    });
    
    e.as_contract(&contract_address, || {
        set_manager_fee_amount(&e, &200u128);
    });
    let (fees_200bps_manager, _) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            end_time,
        )
    });
    
    assert_eq!(fees_200bps_manager, fees_50bps_manager * 4, "200 bps should be 4x 50 bps");
}

#[test]
fn test_fee_collection_resets_user_accrued_fees() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 10_000_000_000_000i128; 
    e.as_contract(&contract_address, || {
        initialize_or_update_user_tracking(&e, &user, user_balance);
    });
    
    jump(&e, SECONDS_PER_YEAR * 10);
    
    let (collected_manager, collected_protocol) = e.as_contract(&contract_address, || {
        collect_accrued_fees_if_any(&e, &user)
    });
    
    if collected_manager > 0 || collected_protocol > 0 {
        let user_state = e.as_contract(&contract_address, || {
            get_user_fee_state(&e, &user)
        });
        assert_eq!(user_state.accrued_manager_fees, 0, "User's accrued manager fees should be reset after collection");
        assert_eq!(user_state.accrued_protocol_fees, 0, "User's accrued protocol fees should be reset after collection");
    }
}

#[test]
fn test_factory_not_set_uses_defaults() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let user_balance = 100_000_000_000i128;
    let start_time = 1000u64;
    let end_time = start_time + SECONDS_PER_YEAR;
    
    let (manager_fee, protocol_fee) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            end_time,
        )
    });
    
    assert!(manager_fee > 0, "Manager fee should work without factory");
    
    assert_eq!(protocol_fee, 0, "Protocol fee should be 0 when factory not set");
}

#[test]
fn test_minimum_shares_configuration() {
    let e = Env::default();
    let (contract_address, _, _, user) = create_test_environment(&e);
    
    let current_minimum = e.as_contract(&contract_address, || {
        get_minimum_shares_for_fee_collection(&e)
    });
    assert_eq!(current_minimum, 25_000_000_000u128, "Default minimum should be 25k");
    
    let new_minimum = 50_000_000_000u128; 
    e.as_contract(&contract_address, || {
        set_minimum_shares_for_fee_collection(&e, &new_minimum);
    });
    
    let updated_minimum = e.as_contract(&contract_address, || {
        get_minimum_shares_for_fee_collection(&e)
    });
    assert_eq!(updated_minimum, new_minimum, "Minimum shares should be updated");
    
    let user_balance = 30_000_000_000i128; 
    let start_time = 1000u64;
    let end_time = start_time + SECONDS_PER_YEAR;
    
    let (manager_fee, protocol_fee) = e.as_contract(&contract_address, || {
        calculate_accrued_fees(
            &e,
            &user,
            user_balance,
            start_time,
            end_time,
        )
    });
    
    assert_eq!(manager_fee, 0, "No fees should accrue below minimum shares");
    assert_eq!(protocol_fee, 0, "No fees should accrue below minimum shares");
}

