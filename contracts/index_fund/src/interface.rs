use soroban_sdk::{Address, Env, Map, Symbol, Vec};
use types::{
    adapter::AdapterType,
    component::{Component, ComponentAllocation, RebalanceParams, RebalanceStatus, RefactorParams},
    index::{IndexFundInfo, IndexFundMetrics, IndexFundStatus},
    volume::VolumeFeeTier,
};

pub trait IndexFundTrait {
    fn mint(e: Env, user: Address, amount: u128);
    fn redeem(e: Env, user: Address, share_amount: u128);
    fn get_whitelist_status(e: Env, address: Address) -> bool;
    fn get_blacklist_status(e: Env, address: Address) -> bool;
    fn get_component(e: Env, token: Address) -> Component;
    fn get_component_balance(e: Env, token: Address) -> u128;
}

pub trait AdminInterface {
    fn refactor(e: Env, caller: Address, params: RefactorParams);

    fn rebalance(e: Env, caller: Address, params: RebalanceParams);

    //   ________  _______  ___________  ___________  _______   _______    ________
    //  /"       )/"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (:   \___/(: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \___  \   \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //   __/  \\  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    //  /" \   :)(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    // (_______/  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        fee_admin: Address,
    );

    fn set_rebalance_authority(e: Env, admin: Address, authority: Address, status: bool);

    fn set_factory(e: Env, admin: Address, factory: Address);

    fn set_adapter_registry(e: Env, admin: Address, registry: Address);

    fn set_whitelist_status(e: Env, admin: Address, address: Address, status: bool);

    fn set_blacklist_status(e: Env, admin: Address, address: Address, status: bool);

    fn set_rebalance_threshold(e: Env, admin: Address, threshold: u64);

    fn set_trade_fee_tiers(e: Env, admin: Address, tiers: Vec<VolumeFeeTier>);

    fn set_trade_fee_tiers_manager(e: Env, admin: Address, manager_fee_bps: u32);

    fn claim_protocol_fees(e: Env, admin: Address, token: Address, destination: Address) -> u128;

    fn claim_manager_fees(e: Env, admin: Address, token: Address, destination: Address) -> u128;

    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    fn get_factory(e: Env) -> Address;

    // Get map of privileged roles
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>>;
}

// Query Interface
pub trait QueryInterface {
    // Comprehensive index information
    fn get_index_info(e: Env) -> IndexFundInfo;

    // Component and balance queries
    fn get_all_components(e: Env) -> Map<Address, Component>;
    fn get_component_info(e: Env, token: Address) -> Component;
    fn get_all_component_balances(e: Env) -> Map<Address, u128>;

    // Financial metrics
    fn get_index_metrics(e: Env) -> IndexFundMetrics;
    fn get_share_price(e: Env) -> u128;
    fn get_current_nav(e: Env) -> u128;

    // Operational status
    fn get_index_status(e: Env) -> IndexFundStatus;
    fn can_rebalance(e: Env) -> bool;

    // Rebalancing queries
    fn get_rebalance_status(e: Env) -> RebalanceStatus;
    fn can_address_rebalance(e: Env, caller: Address) -> bool;
    fn get_component_allocation(e: Env) -> Map<Address, ComponentAllocation>;
    fn get_rebalance_authorities(e: Env) -> Vec<Address>;
    fn get_user_monthly_volume(e: Env, user: Address) -> u128;
    fn get_trade_fee_tiers(e: Env) -> Vec<VolumeFeeTier>;
}
