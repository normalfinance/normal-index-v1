use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env, String};

pub fn create_index_token_contract(
    e: &Env,
    token_wasm_hash: BytesN<32>,
    token_symbol: &String,
) -> Address {
    let mut salt: Bytes = Bytes::new(e);
    salt.append(&token_symbol.clone().to_xdr(e));
    let salt = e.crypto().sha256(&salt);
    e.deployer()
        .with_current_contract(salt)
        .deploy_v2(token_wasm_hash, ())
}
