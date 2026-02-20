use soroban_sdk::{Address, Env, Map, Symbol, Vec};
use types::{
    component::{Component, ComponentAllocation, RebalanceParams, RebalanceStatus, RefactorParams},
    index::{IndexFundInfo, IndexFundMetrics, IndexFundStatus},
    volume::VolumeFeeTier,
};

/// Core mint/redeem interface for index-fund users.
pub trait IndexFundTrait {
    /// Mints index shares for `user` using `amount` of quote token.
    fn mint(e: Env, user: Address, amount: u128);
    /// Redeems `share_amount` shares for underlying component value.
    fn redeem(e: Env, user: Address, share_amount: u128);
    /// Returns whether an address is currently whitelisted.
    fn get_whitelist_status(e: Env, address: Address) -> bool;
    /// Returns whether an address is currently blacklisted.
    fn get_blacklist_status(e: Env, address: Address) -> bool;
    /// Returns configured component metadata for a token.
    fn get_component(e: Env, token: Address) -> Component;
    /// Returns the stored component token balance held by the index.
    fn get_component_balance(e: Env, token: Address) -> u128;
}

/// Administrative interface for configuration, refactors, and fee claims.
pub trait AdminInterface {
    /// Applies component add/remove/update operations.
    fn refactor(e: Env, caller: Address, params: RefactorParams);

    /// Executes rebalancing swaps according to target allocations.
    fn rebalance(e: Env, caller: Address, params: RebalanceParams);

    //   ________  _______  ___________  ___________  _______   _______    ________
    //  /"       )/"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (:   \___/(: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \___  \   \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //   __/  \\  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    //  /" \   :)(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    // (_______/  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    /// Sets privileged role addresses in a single update.
    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        fee_admin: Address,
    );

    /// Adds or removes a rebalance authority.
    fn set_rebalance_authority(e: Env, admin: Address, authority: Address, status: bool);

    /// Sets the originating factory contract address.
    fn set_factory(e: Env, admin: Address, factory: Address);

    /// Sets the adapter registry contract address.
    fn set_adapter_registry(e: Env, admin: Address, registry: Address);

    /// Updates whitelist membership for an address.
    fn set_whitelist_status(e: Env, admin: Address, address: Address, status: bool);

    /// Updates blacklist membership for an address.
    fn set_blacklist_status(e: Env, admin: Address, address: Address, status: bool);

    /// Sets the minimum interval between rebalances.
    fn set_rebalance_threshold(e: Env, admin: Address, threshold: u64);

    /// Replaces configured fee tiers.
    fn set_trade_fee_tiers(e: Env, admin: Address, tiers: Vec<VolumeFeeTier>);

    /// Updates manager fee bps across all fee tiers.
    fn set_trade_fee_tiers_manager(e: Env, admin: Address, manager_fee_bps: u32);

    /// Claims accrued protocol fees for `token` into `destination`.
    fn claim_protocol_fees(e: Env, admin: Address, token: Address, destination: Address) -> u128;

    /// Claims accrued manager fees for `token` into `destination`.
    fn claim_manager_fees(e: Env, admin: Address, token: Address, destination: Address) -> u128;

    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    /// Returns the configured factory contract address.
    fn get_factory(e: Env) -> Address;

    /// Returns a role-to-addresses mapping for privileged roles.
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>>;
}

/// Read-only query interface for index state and metrics.
pub trait QueryInterface {
    /// Returns top-level index metadata and configuration.
    fn get_index_info(e: Env) -> IndexFundInfo;

    /// Returns all configured components keyed by token address.
    fn get_all_components(e: Env) -> Map<Address, Component>;
    /// Returns component metadata for a single token.
    fn get_component_info(e: Env, token: Address) -> Component;
    /// Returns all tracked component balances.
    fn get_all_component_balances(e: Env) -> Map<Address, u128>;

    /// Returns aggregate financial metrics.
    fn get_index_metrics(e: Env) -> IndexFundMetrics;
    /// Returns the current share price.
    fn get_share_price(e: Env) -> u128;
    /// Returns the current portfolio NAV.
    fn get_current_nav(e: Env) -> u128;

    /// Returns status flags and timestamps for index operations.
    fn get_index_status(e: Env) -> IndexFundStatus;
    /// Returns whether the index can be rebalanced now.
    fn can_rebalance(e: Env) -> bool;

    /// Returns detailed rebalance readiness and timing state.
    fn get_rebalance_status(e: Env) -> RebalanceStatus;
    /// Returns whether the provided caller may rebalance.
    fn can_address_rebalance(e: Env, caller: Address) -> bool;
    /// Returns each component's current allocation data.
    fn get_component_allocation(e: Env) -> Map<Address, ComponentAllocation>;
    /// Returns addresses currently authorized to rebalance.
    fn get_rebalance_authorities(e: Env) -> Vec<Address>;
    /// Returns the caller's volume tracked for the current month bucket.
    fn get_user_monthly_volume(e: Env, user: Address) -> u128;
    /// Returns configured trade-fee tiers.
    fn get_trade_fee_tiers(e: Env) -> Vec<VolumeFeeTier>;
}
