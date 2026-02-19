use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};

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

//  ___      ___       __        __    _____  ___
// |"  \    /"  |     /""\      |" \  (\"   \|"  \
//  \   \  //   |    /    \     ||  | |.\\   \    |
//  /\\  \/.    |   /' /\  \    |:  | |: \.   \\  |
// |: \.        |  //  __'  \   |.  | |.  \    \. |
// |.  \    /:  | /   /  \\  \  /\  |\|    \    \ |
// |___|\__/|___|(___/    \___)(__\_|_)\___|\____\)

pub(crate) trait FactoryEvents {
    fn index_deployed(
        &self,
        ts: u64,
        manager: Address,
        index: Address,
        sequence: u32,
        name: String,
        symbol: String,
        initial_price: u128,
        is_public: bool,
    );

    fn mint(
        &self,
        user: Address,
        index: Address,
        amount: u128,
        protocol_fee: u128,
        manager_fee: u128,
        ts: u64,
    );

    fn redeem(&self, ts: u64, index: Address, user: Address, share_amount: u128);

    fn rebalance(&self, ts: u64, index: Address, caller: Address);

    fn refactor(&self, ts: u64, index: Address, caller: Address);

    fn claim_system_fees(
        &self,
        ts: u64,
        index: Address,
        caller: Address,
        token: Address,
        amount: u128,
        destination: Address,
    );

    fn claim_manager_fees(
        &self,
        ts: u64,
        index: Address,
        caller: Address,
        token: Address,
        amount: u128,
        destination: Address,
    );
}

impl FactoryEvents for Events {
    // Enhanced event implementations
    fn index_deployed(
        &self,
        ts: u64,
        manager: Address,
        index: Address,
        sequence: u32,
        name: String,
        symbol: String,
        initial_price: u128,
        is_public: bool,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "index_deployed"),
                ts,
                manager.clone(),
                index.clone(),
                sequence,
            ),
            (name, symbol, initial_price, is_public),
        );
    }

    fn mint(&self, user: Address, index: Address, amount: u128, ts: u64) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "mint"), index, user), (amount, ts));
    }

    fn redeem(&self, user: Address, index: Address, share_amount: u128, ts: u64) {
        self.env().events().publish(
            (Symbol::new(self.env(), "redeem"), index, user),
            (share_amount, ts),
        );
    }

    fn rebalance(&self, ts: u64, index: Address, caller: Address) {
        self.env().events().publish(
            (Symbol::new(self.env(), "rebalance"), ts, index, caller),
            (),
        );
    }

    fn refactor(&self, ts: u64, index: Address, caller: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "refactor"), ts, index, caller), ());
    }

    fn claim_system_fees(
        &self,
        ts: u64,
        index: Address,
        caller: Address,
        token: Address,
        amount: u128,
        destination: Address,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "claim_system_fees"),
                ts,
                index,
                caller,
            ),
            (token, amount, destination),
        );
    }

    fn claim_manager_fees(
        &self,
        ts: u64,
        index: Address,
        caller: Address,
        token: Address,
        amount: u128,
        destination: Address,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "claim_manager_fees"),
                ts,
                index,
                caller,
            ),
            (token, amount, destination),
        );
    }
}

//   ______    ______    _____  ___    _______  __     _______
//  /" _  "\  /    " \  (\"   \|"  \  /"     "||" \   /" _   "|
// (: ( \___)// ____  \ |.\\   \    |(: ______)||  | (: ( \___)
//  \/ \    /  /    ) :)|: \.   \\  | \/    |  |:  |  \/ \
//  //  \ _(: (____/ // |.  \    \. | // ___)  |.  |  //  \ ___
// (:   _) \\        /  |    \    \ |(:  (     /\  |\(:   _(  _|
//  \_______)\"_____/    \___|\____\) \__/    (__\_|_)\_______)

pub(crate) trait FactoryConfigEvents {
    fn index_wasm_updated(
        &self,
        ts: u64,
        admin: Address,
        old_wasm: BytesN<32>,
        new_wasm: BytesN<32>,
    );

    fn token_wasm_updated(
        &self,
        ts: u64,
        admin: Address,
        old_wasm: BytesN<32>,
        new_wasm: BytesN<32>,
    );

    fn adapter_registry_updated(
        &self,
        ts: u64,
        admin: Address,
        old_registry: Address,
        new_registry: Address,
    );
}

impl FactoryConfigEvents for Events {
    fn index_wasm_updated(
        &self,
        ts: u64,
        admin: Address,
        old_wasm: BytesN<32>,
        new_wasm: BytesN<32>,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "index_wasm_updated"),
                ts,
                admin,
                old_wasm,
                new_wasm,
            ),
            (),
        );
    }

    fn token_wasm_updated(
        &self,
        ts: u64,
        admin: Address,
        old_wasm: BytesN<32>,
        new_wasm: BytesN<32>,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "token_wasm_updated"),
                ts,
                admin,
                old_wasm,
                new_wasm,
            ),
            (),
        );
    }

    fn adapter_registry_updated(
        &self,
        ts: u64,
        admin: Address,
        old_registry: Address,
        new_registry: Address,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "adapter_registry_updated"),
                ts,
                admin,
                old_registry,
                new_registry,
            ),
            (),
        );
    }
}
