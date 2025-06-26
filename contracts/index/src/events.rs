use soroban_sdk::{ Address, Env, Symbol };

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

    fn kill_deposit(&self);

    fn unkill_deposit(&self);

    fn kill_request_withdraw(&self);

    fn unkill_request_withdraw(&self);

    fn kill_withdraw(&self);

    fn unkill_withdraw(&self);
}

impl IndexEvents for Events {
    fn mint(&self, user: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "mint"), user), ());
    }

    fn redeem(&self, user: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "redeem"), user), ());
    }

    fn rebalance(&self, user: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "rebalance"), user), ());
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
}
