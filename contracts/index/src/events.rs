use soroban_sdk::{Address, Env, Symbol, Vec};

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

    // Revenue Share Events
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

    fn manager_address_updated(&self, old_manager: Address, new_manager: Address);

    fn protocol_fee_recipient_updated(&self, old_recipient: Address, new_recipient: Address);

    fn fee_collection_toggled(&self, enabled: bool);

    // Rebalancing Events
    fn component_added(&self, token: Address, weight: u128);

    fn component_removed(&self, token: Address);

    fn component_weight_updated(&self, token: Address, old_weight: u128, new_weight: u128);

    fn rebalance_authority_updated(&self, authority: Address, status: bool);

    fn rebalance_completed(&self, caller: Address, components_updated: u32, total_swaps: u32);
}

impl IndexEvents for Events {
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

    fn manager_address_updated(&self, old_manager: Address, new_manager: Address) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "manager_address_updated"),
                old_manager,
                new_manager,
            ),
            (),
        )
    }

    fn protocol_fee_recipient_updated(&self, old_recipient: Address, new_recipient: Address) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "protocol_fee_recipient_updated"),
                old_recipient,
                new_recipient,
            ),
            (),
        )
    }

    fn fee_collection_toggled(&self, enabled: bool) {
        self.env().events().publish(
            (Symbol::new(self.env(), "fee_collection_toggled"), enabled),
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
