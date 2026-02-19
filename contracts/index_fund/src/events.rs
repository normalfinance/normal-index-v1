use soroban_sdk::{Address, Env, Map, Symbol, Vec};
use types::component::Component;

#[derive(Clone)]
pub(crate) struct Events(Env);

impl Events {
    #[inline(always)]
    pub(crate) fn env(&self) -> &Env {
        &self.0
    }

    #[inline(always)]
    pub(crate) fn new(env: &Env) -> Events {
        Events(env.clone())
    }
}

pub(crate) trait IndexEvents {
    fn mint(
        &self,
        ts: u64,
        user: Address,
        token_in: Address,
        amount_in: u128,
        shares_minted: u128,
        share_price: u128,
        nav_before: u128,
        nav_after: u128,
        total_shares_before: u128,
        total_shares_after: u128,
        protocol_fee: u128,
        manager_fee: u128,
    );

    fn redemption(
        &self,
        ts: u64,
        user: Address,
        shares_redeemed: u128,
        share_price: u128,
        nav_before: u128,
        nav_after: u128,
        total_shares_before: u128,
        total_shares_after: u128,
        component_payouts: Map<Address, u128>,
    );

    fn rebalance(
        &self,
        ts: u64,
        caller: Address,
        nav_before: u128,
        nav_after: u128,
        components_before: Map<Address, Component>,
        components_after: Map<Address, Component>,
        total_swaps: u32,
    );

    fn refactor(
        &self,
        ts: u64,
        caller: Address,
        components_before: Map<Address, Component>,
        components_after: Map<Address, Component>,
        components_updated: u32,
    );

    fn swap(
        &self,
        tokens: Vec<Address>,
        user: Address,
        adapter: Symbol,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        amount_out: u128,
    );

    fn swap_failed(
        &self,
        user: Address,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        error_code: u32,
    );

    fn whitelist_status_updated(
        &self,
        ts: u64,
        admin: Address,
        user: Address,
        old_status: bool,
        new_status: bool,
    );

    fn blacklist_status_updated(
        &self,
        ts: u64,
        admin: Address,
        user: Address,
        old_status: bool,
        new_status: bool,
    );

    // Enhanced component management events
    fn component_added(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        weight: u128,
        initial_balance: u128,
        nav_impact: u128,
    );

    fn component_removed(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        final_balance: u128,
        proceeds_distributed: u128,
        nav_impact: u128,
    );

    fn component_weight_updated(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        old_weight: u128,
        new_weight: u128,
        balance_before: u128,
        balance_after: u128,
        nav_impact: u128,
    );

    fn component_oracle_updated(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        old_oracle: Address,
        new_oracle: Address,
    );

    fn component_adapter_updated(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        old_adapter: Symbol,
        new_adapter: Symbol,
    );

    fn rebalance_authority_updated(
        &self,
        ts: u64,
        admin: Address,
        authority: Address,
        old_status: bool,
        new_status: bool,
    );

    fn rebalance_completed(
        &self,
        ts: u64,
        caller: Address,
        components_updated: u32,
        total_swaps: u32,
        performance_delta: i128,
        nav_before: u128,
        nav_after: u128,
        duration_ms: u64,
    );
}

impl IndexEvents for Events {
    fn mint(
        &self,
        ts: u64,
        user: Address,
        token_in: Address,
        amount_in: u128,
        shares_minted: u128,
        share_price: u128,
        nav_before: u128,
        nav_after: u128,
        total_shares_before: u128,
        total_shares_after: u128,
        protocol_fee: u128,
        manager_fee: u128,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "mint"), ts, user.clone()),
            (
                token_in,
                amount_in,
                shares_minted,
                share_price,
                nav_before,
                nav_after,
                total_shares_before,
                total_shares_after,
                protocol_fee,
                manager_fee,
            ),
        );
    }

    fn redemption(
        &self,
        ts: u64,
        user: Address,
        shares_redeemed: u128,
        share_price: u128,
        nav_before: u128,
        nav_after: u128,
        total_shares_before: u128,
        total_shares_after: u128,
        component_payouts: Map<Address, u128>,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "redemption"), ts, user),
            (
                shares_redeemed,
                share_price,
                nav_before,
                nav_after,
                total_shares_before,
                total_shares_after,
                component_payouts,
            ),
        );
    }

    fn rebalance(
        &self,
        ts: u64,
        caller: Address,
        nav_before: u128,
        nav_after: u128,
        components_before: Map<Address, Component>,
        components_after: Map<Address, Component>,
        total_swaps: u32,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "rebalance"), ts, caller),
            (
                nav_before,
                nav_after,
                components_before,
                components_after,
                total_swaps,
            ),
        );
    }

    fn refactor(
        &self,
        ts: u64,
        caller: Address,
        components_before: Map<Address, Component>,
        components_after: Map<Address, Component>,
        components_updated: u32,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "refactor"), ts, caller),
            (components_before, components_after, components_updated),
        );
    }

    fn whitelist_status_updated(
        &self,
        ts: u64,
        admin: Address,
        user: Address,
        old_status: bool,
        new_status: bool,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "whitelist_status_updated"),
                ts,
                admin,
                user,
                old_status,
                new_status,
            ),
            (),
        );
    }

    fn blacklist_status_updated(
        &self,
        ts: u64,
        admin: Address,
        user: Address,
        old_status: bool,
        new_status: bool,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "blacklist_status_updated"),
                ts,
                admin,
                user,
                old_status,
                new_status,
            ),
            (),
        );
    }

    fn component_added(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        weight: u128,
        initial_balance: u128,
        nav_impact: u128,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "component_added"), ts, admin, token),
            (weight, initial_balance, nav_impact),
        );
    }

    fn component_removed(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        final_balance: u128,
        proceeds_distributed: u128,
        nav_impact: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "component_removed"),
                ts,
                admin,
                token,
            ),
            (final_balance, proceeds_distributed, nav_impact),
        );
    }

    fn component_weight_updated(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        old_weight: u128,
        new_weight: u128,
        balance_before: u128,
        balance_after: u128,
        nav_impact: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "component_weight_updated"),
                ts,
                admin,
                token,
            ),
            (
                old_weight,
                new_weight,
                balance_before,
                balance_after,
                nav_impact,
            ),
        );
    }

    fn component_oracle_updated(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        old_oracle: Address,
        new_oracle: Address,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "component_oracle_updated"),
                ts,
                admin,
                token,
            ),
            (old_oracle, new_oracle),
        );
    }

    fn component_adapter_updated(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        old_adapter: Symbol,
        new_adapter: Symbol,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "component_adapter_updated"),
                ts,
                admin,
                token,
            ),
            (old_adapter, new_adapter),
        );
    }

    fn rebalance_authority_updated(
        &self,
        ts: u64,
        admin: Address,
        authority: Address,
        old_status: bool,
        new_status: bool,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "rebalance_authority_updated"),
                ts,
                admin,
                authority,
            ),
            (old_status, new_status),
        );
    }

    fn rebalance_completed(
        &self,
        ts: u64,
        caller: Address,
        components_updated: u32,
        total_swaps: u32,
        performance_delta: i128,
        nav_before: u128,
        nav_after: u128,
        duration_ms: u64,
    ) {
        self.env().events().publish(
            (Symbol::new(self.env(), "rebalance_completed"), ts, caller),
            (
                components_updated,
                total_swaps,
                performance_delta,
                nav_before,
                nav_after,
                duration_ms,
            ),
        );
    }

    fn swap(
        &self,
        tokens: Vec<Address>,
        user: Address,
        adapter: Symbol,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        amount_out: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "swap"),
                tokens,
                user,
                adapter,
                token_in,
                token_out,
                amount_in,
                amount_out,
            ),
            (),
        );
    }

    fn swap_failed(
        &self,
        user: Address,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        error_code: u32,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "swap_failed"),
                user,
                token_in,
                token_out,
                amount_in,
                error_code,
            ),
            (),
        );
    }
}
