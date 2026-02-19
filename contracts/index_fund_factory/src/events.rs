use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

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
        deployer: Address,
        index_address: Address,
        operator: Address,
        manager: Address,
        initial_components: Vec<Address>,
        initial_weights: Vec<u128>,
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
        deployer: Address,
        index_address: Address,
        operator: Address,
        manager: Address,
        initial_components: Vec<Address>,
        initial_weights: Vec<u128>,
        initial_price: u128,
        is_public: bool,
    ) {
        // Split into multiple events due to Soroban topic limits
        // Primary deployment event
        self.env().events().publish(
            (
                Symbol::new(self.env(), "index_deployed"),
                ts,
                deployer.clone(),
                index_address.clone(),
                operator,
                manager,
            ),
            (),
        );

        // Configuration event
        self.env().events().publish(
            (
                Symbol::new(self.env(), "index_config"),
                ts,
                index_address.clone(),
                initial_price,
                is_public,
            ),
            (),
        );

        // Components event
        self.env().events().publish(
            (
                Symbol::new(self.env(), "index_components"),
                ts,
                index_address,
                initial_components,
                initial_weights,
            ),
            (),
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
        version: u32,
    );

    fn token_wasm_updated(
        &self,
        ts: u64,
        admin: Address,
        old_wasm: BytesN<32>,
        new_wasm: BytesN<32>,
        version: u32,
    );
}

impl FactoryConfigEvents for Events {
    fn index_wasm_updated(
        &self,
        ts: u64,
        admin: Address,
        old_wasm: BytesN<32>,
        new_wasm: BytesN<32>,
        version: u32,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "wasm_updated"),
                ts,
                admin,
                old_wasm,
                new_wasm,
                version,
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
        version: u32,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "wasm_updated"),
                ts,
                admin,
                old_wasm,
                new_wasm,
                version,
            ),
            (),
        );
    }
}
