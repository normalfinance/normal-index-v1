#![cfg(test)]
extern crate std;

use crate::testutils::{Setup, TestConfig};
use soroban_sdk::{testutils::Address as _, Address};

#[test]
fn test_mint() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let mint_amount = 100_0000000;

    assert_eq!(
        setup
            .provider
            .get_protocol_fees_by_token(&setup.token_usdc.address),
        0
    );

    setup.provider.mint(
        &user1,
        &setup.index_fund,
        &setup.token_usdc.address,
        &mint_amount,
        &None,
    );

    // protocol fee increased
    let (mint_fee, _) = setup.provider.get_fee_config();

    assert_eq!(
        setup
            .provider
            .get_protocol_fees_by_token(&setup.token_usdc.address),
        mint_fee
    );
}

#[test]
fn test_redeem() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    let user1 = setup.users[1].clone();

    setup.provider.redeem(
        &user1,
        &setup.index_fund,
        &setup.token_usdc.address,
        &100_0000000,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #101)")]
fn test_set_invalid_mint_fee() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    setup.provider.set_mint_fee(&setup.admin, &100000);
}

#[test]
#[should_panic(expected = "Error(Contract, #101)")]
fn test_set_invalid_redeem_fee() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    setup.provider.set_redeem_fee(&setup.admin, &100000);
}

#[test]
#[should_panic(expected = "Error(Contract, #100)")]
fn test_set_index_fund_call_failure() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    let user1 = setup.users[1].clone();
    let bogus_index_fund = Address::generate(&setup.env);

    setup.provider.mint(
        &user1,
        &bogus_index_fund,
        &setup.token_usdc.address,
        &100_0000000,
        &None,
    );
}

#[test]
fn test_claim_protocol_fees() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    assert_eq!(setup.token_usdc.balance(&setup.fee_destination), 0);

    let claimed = setup.provider.claim_protocol_fees(
        &setup.admin,
        &setup.token_usdc.address,
        &setup.fee_destination,
    );

    assert_eq!(
        setup
            .provider
            .get_protocol_fees_by_token(&setup.token_usdc.address),
        0
    );

    assert_eq!(
        setup.token_usdc.balance(&setup.fee_destination),
        claimed as i128
    );
}
