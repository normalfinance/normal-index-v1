use soroban_sdk::{Address, Env, Symbol};

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

pub(crate) trait IndexFundFeeProviderEvents {
    fn mint(&self, user: Address, index_fund: Address, token: Address, amount: u128);

    fn redeem(&self, user: Address, index_fund: Address, token: Address, share_amount: u128);

    fn charge_provider_fee(&self, token: Address, amount: u128);

    fn claim_fee(&self, token: Address, amount: u128, to: Address);

    fn set_mint_fee(&self, fee: u128);

    fn set_redeem_fee(&self, fee: u128);

    fn kill_fee(&self);

    fn unkill_fee(&self);
}

impl IndexFundFeeProviderEvents for Events {
    fn mint(&self, user: Address, index_fund: Address, token: Address, amount: u128) {
        self.env().events().publish(
            (Symbol::new(self.env(), "mint"), index_fund, user),
            (token, amount),
        );
    }

    fn redeem(&self, user: Address, index_fund: Address, token: Address, share_amount: u128) {
        self.env().events().publish(
            (Symbol::new(self.env(), "redeem"), index_fund, user),
            (token, share_amount),
        );
    }

    fn charge_provider_fee(&self, token: Address, amount: u128) {
        self.env().events().publish(
            (Symbol::new(self.env(), "charge_provider_fee"),),
            (token, amount),
        );
    }

    fn claim_fee(&self, token: Address, amount: u128, to: Address) {
        self.env().events().publish(
            (Symbol::new(self.env(), "withdraw_fee"),),
            (token, amount, to),
        );
    }

    fn set_mint_fee(&self, fee: u128) {
        let e = self.env();
        e.events()
            .publish((Symbol::new(e, "set_mint_fee"),), (fee,));
    }

    fn set_redeem_fee(&self, fee: u128) {
        let e = self.env();
        e.events()
            .publish((Symbol::new(e, "set_redeem_fee"),), (fee,));
    }

    fn kill_fee(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "kill_fee"),), ())
    }

    fn unkill_fee(&self) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "unkill_fee"),), ())
    }
}
