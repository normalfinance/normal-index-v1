use crate::storage::Component;
use soroban_sdk::{Address, Env, Map, Symbol, Vec};

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
    // Enhanced mint event with comprehensive data for analytics
    fn mint_executed(
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
        fees_collected: u128,
        destination: Option<Address>,
    );

    // Enhanced redemption event with component breakdown
    fn redemption_executed(
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
        fees_deducted: u128,
    );

    // Enhanced rebalancing event with detailed swap and performance data
    fn rebalance_executed(
        &self,
        ts: u64,
        caller: Address,
        nav_before: u128,
        nav_after: u128,
        components_before: Map<Address, Component>,
        components_after: Map<Address, Component>,
        total_swaps: u32,
        gas_cost: u128,
        performance_impact: i128, // Can be negative if rebalancing reduced NAV
    );

    // Legacy events for backward compatibility
    fn mint(&self, ts: u64, user: Address);

    fn redeem(&self, ts: u64, user: Address);

    fn rebalance(&self, ts: u64, user: Address);

    fn swap(
        &self,
        tokens: Vec<Address>,
        user: Address,
        pool_id: Symbol,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        amount_out: i128,
    );

    fn kill_deposit(&self);

    fn unkill_deposit(&self);

    fn kill_request_withdraw(&self);

    fn unkill_request_withdraw(&self);

    fn kill_withdraw(&self);

    fn unkill_withdraw(&self);

    // Enhanced fee collection events with user tracking and periods
    fn accrued_fees_collected(
        &self,
        ts: u64,
        user: Address,
        shares_before: u128,
        shares_after: u128,
        fee_period_start: u64,
        fee_period_end: u64,
        annual_fee_rate: u32,
        total_fee_collected: u128,
        manager_fee_portion: u128,
        protocol_fee_portion: u128,
    );

    fn fees_distributed_to_manager(
        &self,
        ts: u64,
        manager: Address,
        amount: u128,
        total_accumulated_before: u128,
        total_accumulated_after: u128,
    );

    fn fees_distributed_to_protocol(
        &self,
        ts: u64,
        recipient: Address,
        amount: u128,
        total_accumulated_before: u128,
        total_accumulated_after: u128,
    );

    // Enhanced configuration update events
    fn manager_address_updated(
        &self,
        ts: u64,
        admin: Address,
        old_manager: Address,
        new_manager: Address,
    );

    fn protocol_fee_recipient_updated(
        &self,
        ts: u64,
        admin: Address,
        old_recipient: Address,
        new_recipient: Address,
    );

    fn manager_fee_fraction_updated(
        &self,
        ts: u64,
        admin: Address,
        old_fee_fraction: u32,
        new_fee_fraction: u32,
    );

    fn public_status_updated(&self, ts: u64, admin: Address, old_status: bool, new_status: bool);

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

    fn rebalance_threshold_updated(
        &self,
        ts: u64,
        admin: Address,
        old_threshold: u64,
        new_threshold: u64,
    );

    fn base_nav_updated(&self, ts: u64, admin: Address, old_nav: u128, new_nav: u128);

    fn initial_price_updated(&self, ts: u64, admin: Address, old_price: u128, new_price: u128);

    // Kill switch events
    fn operation_killed(
        &self,
        ts: u64,
        admin: Address,
        operation: Symbol, // "mint", "redeem", "rebalance"
    );

    fn operation_unkilled(
        &self,
        ts: u64,
        admin: Address,
        operation: Symbol, // "mint", "redeem", "rebalance"
    );

    // Enhanced component management events
    fn component_added_detailed(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        weight: u128,
        initial_balance: u128,
        nav_impact: u128,
    );

    fn component_removed_detailed(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        final_balance: u128,
        proceeds_distributed: u128,
        nav_impact: u128,
    );

    fn component_weight_updated_detailed(
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

    fn rebalance_authority_updated_detailed(
        &self,
        ts: u64,
        admin: Address,
        authority: Address,
        old_status: bool,
        new_status: bool,
    );

    // Enhanced rebalance completion event
    fn rebalance_completed_detailed(
        &self,
        ts: u64,
        caller: Address,
        components_updated: u32,
        total_swaps: u32,
        total_gas_cost: u128,
        performance_delta: i128,
        nav_before: u128,
        nav_after: u128,
        duration_ms: u64,
    );

    // Legacy Revenue Share Events (for backward compatibility)
    fn fee_collected(
        &self,
        user: Address,
        token: Address,
        amount: u128,
        manager_fee: u128,
        protocol_fee: u128,
    );

    fn manager_fees_distributed(&self, manager: Address, amount: u128);

    fn protocol_fees_distributed(&self, recipient: Address, amount: u128);

    fn manager_address_updated_legacy(&self, old_manager: Address, new_manager: Address);

    fn protocol_fee_recipient_updated_legacy(&self, old_recipient: Address, new_recipient: Address);

    // Legacy Rebalancing Events (for backward compatibility)
    fn component_added(&self, token: Address, weight: u128);

    fn component_removed(&self, token: Address);

    fn component_weight_updated(&self, token: Address, old_weight: u128, new_weight: u128);

    fn rebalance_authority_updated(&self, authority: Address, status: bool);

    fn rebalance_completed(&self, caller: Address, components_updated: u32, total_swaps: u32);
}

impl IndexEvents for Events {
    // Enhanced event implementations
    fn mint_executed(
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
        fees_collected: u128,
        destination: Option<Address>,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "mint_executed"),
                ts,
                user.clone(),
                token_in,
                amount_in,
                shares_minted,
                share_price,
                nav_before,
                nav_after,
                total_shares_before,
                total_shares_after,
                fees_collected,
                destination.unwrap_or(user),
            ),
            (),
        );
    }

    fn redemption_executed(
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
        fees_deducted: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "redemption_executed"),
                ts,
                user,
                shares_redeemed,
                share_price,
                nav_before,
                nav_after,
                total_shares_before,
                total_shares_after,
                component_payouts,
                fees_deducted,
            ),
            (),
        );
    }

    fn rebalance_executed(
        &self,
        ts: u64,
        caller: Address,
        nav_before: u128,
        nav_after: u128,
        components_before: Map<Address, Component>,
        components_after: Map<Address, Component>,
        total_swaps: u32,
        gas_cost: u128,
        performance_impact: i128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "rebalance_executed"),
                ts,
                caller,
                nav_before,
                nav_after,
                components_before,
                components_after,
                total_swaps,
                gas_cost,
                performance_impact,
            ),
            (),
        );
    }

    fn accrued_fees_collected(
        &self,
        ts: u64,
        user: Address,
        shares_before: u128,
        shares_after: u128,
        fee_period_start: u64,
        fee_period_end: u64,
        annual_fee_rate: u32,
        total_fee_collected: u128,
        manager_fee_portion: u128,
        protocol_fee_portion: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "accrued_fees_collected"),
                ts,
                user,
                shares_before,
                shares_after,
                fee_period_start,
                fee_period_end,
                annual_fee_rate,
                total_fee_collected,
                manager_fee_portion,
                protocol_fee_portion,
            ),
            (),
        );
    }

    fn fees_distributed_to_manager(
        &self,
        ts: u64,
        manager: Address,
        amount: u128,
        total_accumulated_before: u128,
        total_accumulated_after: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "fees_distributed_to_manager"),
                ts,
                manager,
                amount,
                total_accumulated_before,
                total_accumulated_after,
            ),
            (),
        );
    }

    fn fees_distributed_to_protocol(
        &self,
        ts: u64,
        recipient: Address,
        amount: u128,
        total_accumulated_before: u128,
        total_accumulated_after: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "fees_distributed_to_protocol"),
                ts,
                recipient,
                amount,
                total_accumulated_before,
                total_accumulated_after,
            ),
            (),
        );
    }

    // Configuration update event implementations
    fn manager_address_updated(
        &self,
        ts: u64,
        admin: Address,
        old_manager: Address,
        new_manager: Address,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "manager_address_updated"),
                ts,
                admin,
                old_manager,
                new_manager,
            ),
            (),
        );
    }

    fn protocol_fee_recipient_updated(
        &self,
        ts: u64,
        admin: Address,
        old_recipient: Address,
        new_recipient: Address,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "protocol_fee_recipient_updated"),
                ts,
                admin,
                old_recipient,
                new_recipient,
            ),
            (),
        );
    }

    fn manager_fee_fraction_updated(
        &self,
        ts: u64,
        admin: Address,
        old_fee_fraction: u32,
        new_fee_fraction: u32,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "manager_fee_fraction_updated"),
                ts,
                admin,
                old_fee_fraction,
                new_fee_fraction,
            ),
            (),
        );
    }

    fn public_status_updated(&self, ts: u64, admin: Address, old_status: bool, new_status: bool) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "public_status_updated"),
                ts,
                admin,
                old_status,
                new_status,
            ),
            (),
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

    fn rebalance_threshold_updated(
        &self,
        ts: u64,
        admin: Address,
        old_threshold: u64,
        new_threshold: u64,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "rebalance_threshold_updated"),
                ts,
                admin,
                old_threshold,
                new_threshold,
            ),
            (),
        );
    }

    fn base_nav_updated(&self, ts: u64, admin: Address, old_nav: u128, new_nav: u128) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "base_nav_updated"),
                ts,
                admin,
                old_nav,
                new_nav,
            ),
            (),
        );
    }

    fn initial_price_updated(&self, ts: u64, admin: Address, old_price: u128, new_price: u128) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "initial_price_updated"),
                ts,
                admin,
                old_price,
                new_price,
            ),
            (),
        );
    }

    fn operation_killed(&self, ts: u64, admin: Address, operation: Symbol) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "operation_killed"),
                ts,
                admin,
                operation,
            ),
            (),
        );
    }

    fn operation_unkilled(&self, ts: u64, admin: Address, operation: Symbol) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "operation_unkilled"),
                ts,
                admin,
                operation,
            ),
            (),
        );
    }

    // Enhanced component management event implementations
    fn component_added_detailed(
        &self,
        ts: u64,
        admin: Address,
        token: Address,
        weight: u128,
        initial_balance: u128,
        nav_impact: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "component_added_detailed"),
                ts,
                admin,
                token,
                weight,
                initial_balance,
                nav_impact,
            ),
            (),
        );
    }

    fn component_removed_detailed(
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
                Symbol::new(self.env(), "component_removed_detailed"),
                ts,
                admin,
                token,
                final_balance,
                proceeds_distributed,
                nav_impact,
            ),
            (),
        );
    }

    fn component_weight_updated_detailed(
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
                Symbol::new(self.env(), "component_weight_updated_detailed"),
                ts,
                admin,
                token,
                old_weight,
                new_weight,
                balance_before,
                balance_after,
                nav_impact,
            ),
            (),
        );
    }

    fn rebalance_authority_updated_detailed(
        &self,
        ts: u64,
        admin: Address,
        authority: Address,
        old_status: bool,
        new_status: bool,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "rebalance_authority_updated_detailed"),
                ts,
                admin,
                authority,
                old_status,
                new_status,
            ),
            (),
        );
    }

    fn rebalance_completed_detailed(
        &self,
        ts: u64,
        caller: Address,
        components_updated: u32,
        total_swaps: u32,
        total_gas_cost: u128,
        performance_delta: i128,
        nav_before: u128,
        nav_after: u128,
        duration_ms: u64,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "rebalance_completed_detailed"),
                ts,
                caller,
                components_updated,
                total_swaps,
                total_gas_cost,
                performance_delta,
                nav_before,
                nav_after,
                duration_ms,
            ),
            (),
        );
    }

    // Legacy event implementations for backward compatibility
    fn mint(&self, ts: u64, user: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "mint"), ts, user), ());
    }

    fn redeem(&self, ts: u64, user: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "redeem"), ts, user), ());
    }

    fn rebalance(&self, ts: u64, user: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "rebalance"), ts, user), ());
    }

    fn swap(
        &self,
        tokens: Vec<Address>,
        user: Address,
        pool_id: Symbol,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        amount_out: i128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "swap"),
                tokens,
                user,
                pool_id,
                token_in,
                token_out,
                amount_in,
                amount_out,
            ),
            (),
        );
    }

    fn kill_deposit(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_deposit"),), ())
    }

    fn unkill_deposit(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_deposit"),), ())
    }

    fn kill_request_withdraw(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_request_withdraw"),), ())
    }

    fn unkill_request_withdraw(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_request_withdraw"),), ())
    }

    fn kill_withdraw(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_withdraw"),), ())
    }

    fn unkill_withdraw(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_withdraw"),), ())
    }

    // Revenue Share Event Implementations
    fn fee_collected(
        &self,
        user: Address,
        token: Address,
        amount: u128,
        manager_fee: u128,
        protocol_fee: u128,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "fee_collected"),
                user,
                token,
                amount,
                manager_fee,
                protocol_fee,
            ),
            (),
        )
    }

    fn manager_fees_distributed(&self, manager: Address, amount: u128) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "manager_fees_distributed"),
                manager,
                amount,
            ),
            (),
        )
    }

    fn protocol_fees_distributed(&self, recipient: Address, amount: u128) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "protocol_fees_distributed"),
                recipient,
                amount,
            ),
            (),
        )
    }

    fn manager_address_updated_legacy(&self, old_manager: Address, new_manager: Address) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "manager_address_updated"),
                old_manager,
                new_manager,
            ),
            (),
        )
    }

    fn protocol_fee_recipient_updated_legacy(
        &self,
        old_recipient: Address,
        new_recipient: Address,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "protocol_fee_recipient_updated"),
                old_recipient,
                new_recipient,
            ),
            (),
        )
    }

    fn component_added(&self, token: Address, weight: u128) {
        self.env().events().publish(
            (Symbol::new(self.env(), "component_added"), token, weight),
            (),
        )
    }

    fn component_removed(&self, token: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "component_removed"), token), ())
    }

    fn component_weight_updated(&self, token: Address, old_weight: u128, new_weight: u128) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "component_weight_updated"),
                token,
                old_weight,
                new_weight,
            ),
            (),
        )
    }

    fn rebalance_authority_updated(&self, authority: Address, status: bool) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "rebalance_authority_updated"),
                authority,
                status,
            ),
            (),
        )
    }

    fn rebalance_completed(&self, caller: Address, components_updated: u32, total_swaps: u32) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "rebalance_completed"),
                caller,
                components_updated,
                total_swaps,
            ),
            (),
        )
    }
}
