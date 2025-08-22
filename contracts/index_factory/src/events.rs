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
    // Enhanced index deployment event with comprehensive metadata
    fn index_deployed(
        &self,
        ts: u64,
        deployer: Address,
        index_address: Address,
        operator: Address,
        manager: Address,
        fee_destination: Address,
        max_swap_fee_fraction: u32,
        initial_components: Vec<Address>,
        initial_weights: Vec<u128>,
        base_nav: u128,
        initial_price: u128,
        is_public: bool,
        deployment_cost: u128,
    );

    // Factory configuration events
    fn protocol_fee_updated(&self, ts: u64, admin: Address, old_fee: u32, new_fee: u32);

    fn max_management_fee_updated(
        &self,
        ts: u64,
        admin: Address,
        old_max_fee: u32,
        new_max_fee: u32,
    );

    fn factory_admin_updated(&self, ts: u64, old_admin: Address, new_admin: Address);

    fn factory_paused(&self, ts: u64, admin: Address);

    fn factory_unpaused(&self, ts: u64, admin: Address);
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
        fee_destination: Address,
        max_swap_fee_fraction: u32,
        initial_components: Vec<Address>,
        initial_weights: Vec<u128>,
        base_nav: u128,
        initial_price: u128,
        is_public: bool,
        deployment_cost: u128,
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
                fee_destination,
            ),
            (),
        );

        // Configuration event
        self.env().events().publish(
            (
                Symbol::new(self.env(), "index_config"),
                ts,
                index_address.clone(),
                max_swap_fee_fraction,
                base_nav,
                initial_price,
                is_public,
                deployment_cost,
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

    fn protocol_fee_updated(&self, ts: u64, admin: Address, old_fee: u32, new_fee: u32) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "protocol_fee_updated"),
                ts,
                admin,
                old_fee,
                new_fee,
            ),
            (),
        );
    }

    fn max_management_fee_updated(
        &self,
        ts: u64,
        admin: Address,
        old_max_fee: u32,
        new_max_fee: u32,
    ) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "max_management_fee_updated"),
                ts,
                admin,
                old_max_fee,
                new_max_fee,
            ),
            (),
        );
    }

    fn factory_admin_updated(&self, ts: u64, old_admin: Address, new_admin: Address) {
        self.env().events().publish(
            (
                Symbol::new(self.env(), "factory_admin_updated"),
                ts,
                old_admin,
                new_admin,
            ),
            (),
        );
    }

    fn factory_paused(&self, ts: u64, admin: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "factory_paused"), ts, admin), ());
    }

    fn factory_unpaused(&self, ts: u64, admin: Address) {
        self.env()
            .events()
            .publish((Symbol::new(self.env(), "factory_unpaused"), ts, admin), ());
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
  
    fn index_fee_toggled(&self, index_address: Address, enabled: bool);
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

    fn index_fee_toggled(&self, index_address: Address, enabled: bool) {
        self.env().events().publish(
            (Symbol::new(self.env(), "index_fee_toggled"),),
            (index_address, enabled),
        );
    }
}
