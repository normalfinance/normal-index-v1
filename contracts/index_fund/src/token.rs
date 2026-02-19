use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

pub fn create_contract(e: &Env, index_token_wasm_hash: BytesN<32>, sequence: &u32) -> Address {
    let mut salt = Bytes::new(e);
    salt.append(&sequence.to_xdr(e));
    let salt = e.crypto().sha256(&salt);
    e.deployer()
        .with_current_contract(salt)
        .deploy_v2(index_token_wasm_hash, ())
}
