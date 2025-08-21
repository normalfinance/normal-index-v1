use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::interface::DexProvider;

const SWAP_EXECUTED: Symbol = symbol_short!("swap_exec");
const SWAP_FAILED: Symbol = symbol_short!("swap_fail");
const PROVIDER_CONFIG_SET: Symbol = symbol_short!("prov_cfg");

pub(crate) trait SwapEvents {
    fn swap_executed(
        &self,
        provider: DexProvider,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        amount_out: u128,
        user: Address,
    );

    fn swap_failed(
        &self,
        provider: DexProvider,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        error_code: u32,
    );

    fn provider_config_set(&self, provider: DexProvider, contract_address: Address, admin: Address);
}

pub struct Events {
    env: Env,
}

impl Events {
    pub fn new(env: &Env) -> Events {
        Events { env: env.clone() }
    }
}

impl SwapEvents for Events {
    fn swap_executed(
        &self,
        provider: DexProvider,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        amount_out: u128,
        user: Address,
    ) {
        let topics = (SWAP_EXECUTED, provider, token_in, token_out);
        let data = (amount_in, amount_out, user);
        self.env.events().publish(topics, data);
    }

    fn swap_failed(
        &self,
        provider: DexProvider,
        token_in: Address,
        token_out: Address,
        amount_in: u128,
        error_code: u32,
    ) {
        let topics = (SWAP_FAILED, provider, token_in, token_out);
        let data = (amount_in, error_code);
        self.env.events().publish(topics, data);
    }

    fn provider_config_set(
        &self,
        provider: DexProvider,
        contract_address: Address,
        admin: Address,
    ) {
        let topics = (PROVIDER_CONFIG_SET, provider, admin);
        let data = contract_address;
        self.env.events().publish(topics, data);
    }
}
