use soroban_sdk::{Address, Bytes, BytesN, Env, Map, Symbol, Vec};
use types::{
    component::{RebalanceParams, RefactorParams},
    index::IndexParams,
};

use crate::storage::FactoryConfig;

pub trait IndexFundFactoryTrait {
    fn deploy_index_contract(e: Env, serialized_asset: Bytes, params: IndexParams) -> Address;
    fn mint(e: Env, user: Address, index: Address, amount: u128);
    fn redeem(e: Env, user: Address, index: Address, share_amount: u128);
    fn rebalance(e: Env, caller: Address, index: Address, params: RebalanceParams);
    fn refactor(e: Env, caller: Address, index: Address, params: RefactorParams);
    fn claim_system_fees(
        e: Env,
        caller: Address,
        index: Address,
        token: Address,
        destination: Address,
    ) -> u128;
    fn claim_manager_fees(
        e: Env,
        caller: Address,
        index: Address,
        token: Address,
        destination: Address,
    ) -> u128;
}

pub trait AdminInterface {
    // Set privileged addresses
    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        fee_admin: Address,
    );

    // Get map of privileged roles
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>>;

    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    fn get_factory_config(e: Env) -> FactoryConfig;

    fn get_index_contract_wasm(e: Env) -> BytesN<32>;

    fn get_deployed_indexes(e: Env, operator: Address) -> Vec<Address>;

    fn get_all_deployed_indexes(e: Env) -> Vec<Address>;

    fn get_index_count(e: Env, operator: Address) -> u32;

    fn get_total_index_count(e: Env) -> u32;

    //   ________  _______  ___________  ___________  _______   _______    ________
    //  /"       )/"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (:   \___/(: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \___  \   \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //   __/  \\  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    //  /" \   :)(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    // (_______/  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    fn set_index_contract_wasm(e: Env, admin: Address, index_contract_wasm: BytesN<32>);
}
