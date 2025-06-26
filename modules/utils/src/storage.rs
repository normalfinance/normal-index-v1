use soroban_sdk::{ contracttype, Address, BytesN, String, Symbol, Vec };

#[contracttype]
#[derive(Clone)]
pub struct TokenInitInfo {
    // The hash of the liquidity pool token contract.
    pub token_wasm_hash: BytesN<32>,
    pub name: String,
    pub symbol: String,
}

#[contracttype]
#[derive(Clone)]
pub struct PrivilegedAddresses {
    pub emergency_admin: Address,
    pub rewards_admin: Address,
    pub operations_admin: Address,
    pub pause_admin: Address,
    pub emergency_pause_admins: Vec<Address>,
}

#[contracttype]
#[derive(Clone)]
pub struct InitializeParams {
    pub admin: Address,
    pub privileged_addrs: PrivilegedAddresses,
}
