use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

/// Deploys a new index token contract using `sequence` as deterministic salt input.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `index_token_wasm_hash` (`BytesN<32>`): WASM hash of the token contract to deploy.
/// - `sequence` (`&u32`): Factory sequence used to derive deterministic deployment salt.
///
/// # Returns
/// - `Address`: Address of the deployed token contract.
pub fn create_contract(e: &Env, index_token_wasm_hash: BytesN<32>, sequence: &u32) -> Address {
    let mut salt = Bytes::new(e);
    salt.append(&sequence.to_xdr(e));
    let salt = e.crypto().sha256(&salt);
    e.deployer()
        .with_current_contract(salt)
        .deploy_v2(index_token_wasm_hash, ())
}
