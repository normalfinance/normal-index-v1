#![cfg(test)]
extern crate std;

use crate::testutils;
use crate::testutils::long_short_pair::PairTokens;
use crate::testutils::Setup;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, String, Symbol, Vec};
use types::component::ComponentUpdate;
use types::index::{DeployIndexParams, IndexFundAuthorities};
use types::pair::PairParams;

#[test]
fn test_deploy_contract() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let usdc = Address::generate(&e);

    let params = DeployIndexParams {
        authorities: IndexFundAuthorities {
            admin,
            emergency_admin: setup.emergency_admin,
            fee_admin: setup.fee_admin,
            rewards_admin: setup.rewards_admin,
            operations_admin: setup.operations_admin,
            rebalance_authorities: Vec::from_array(&setup.env, []),
        },
        quote_token: Address::generate(&e),
        name: String::from("Top 5 Index"),
        description: String::from("My amazing test index fund"),
        symbol: String::from("TOP5"),
        is_public: true,
        initial_price: 100_0000000,
        components: Vec::from_array(
            &e,
            [ComponentUpdate {
                token: usdc,
                new_weight: 1,
                action: types::component::ComponentAction::Add,
                oracle: Option(Address::generate(&e)),
                adapter: Symbol::from("Normal"),
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
