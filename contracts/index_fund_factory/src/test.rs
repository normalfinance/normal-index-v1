#![cfg(test)]
extern crate std;

use crate::testutils;
use crate::testutils::Setup;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, String, Symbol, Vec};
use types::component::ComponentUpdate;
use types::index::{DeployIndexParams, IndexFundAuthorities};

#[test]
fn test_deploy_contract() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let usdc = Address::generate(&setup.env);

    let params = DeployIndexParams {
        authorities: IndexFundAuthorities {
            admin,
            emergency_admin: setup.emergency_admin,
            fee_admin: setup.fee_admin,
            rewards_admin: setup.rewards_admin,
            operations_admin: setup.operations_admin,
            rebalance_authorities: Vec::from_array(&setup.env, []),
        },
        quote_token: Address::generate(&setup.env),
        name: String::from_str(&setup.env, "Top 5 Index"),
        description: String::from_str(&setup.env, "My amazing test index fund"),
        symbol: String::from_str(&setup.env, "TOP5"),
        is_public: true,
        initial_price: 100_0000000,
        components: Vec::from_array(
            &setup.env,
            [ComponentUpdate {
                token: usdc,
                new_weight: Some(1_u128),
                action: types::component::ComponentAction::Add,
                new_oracle: Some(Address::generate(&setup.env)),
                new_adapter: Some(Symbol::new(&setup.env, "Normal")),
            }],
        ),
    };

    let index_address = setup.factory.deploy_index_contract(&params);

    // Factory assertions
    assert_eq!(setup.factory.get_total_index_count(), 1);
    assert_eq!(setup.factory.get_index_by_id(&1), index_address);
    assert_eq!(setup.factory.get_total_index_count(), 1);

    // Index assertions
    let index_client = testutils::index_fund::Client::new(&setup.env, &index_address);

    assert_eq!(index_client.get_factory(), setup.factory.address);
}
