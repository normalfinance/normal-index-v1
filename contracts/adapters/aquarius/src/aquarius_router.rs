use adapter::AdapterError;
use soroban_sdk::{vec, Address, BytesN, Env, Vec};

soroban_sdk::contractimport!(file = "./wasm/soroban_liquidity_pool_router_contract.wasm");
pub type AquariusRouterClient<'a> = Client<'a>;

/*
        This first version of the AquaAdapter, is written just for pools with 2 tokens, so we will build from
        path = (TokenA, TokenB, TokenC, TokenD)
        bytes = (pool_hash_0, pool_hash_1, pool_hash_2)
        where pool_hash_0 = hash of the pool with tokenA and tokenB
        where pool_hash_1 = hash of the pool with tokenB and tokenC
        where pool_hash_2 = hash of the pool with tokenC and tokenD
        where token_out = tokenD
        where token_in = tokenA
        where in_amount = amount_in
        where out_min = amount_out_min

        The interface is based on https://github.com/AquaToken/soroban-amm/
*/

fn convert_to_swaps_chain(
    e: &Env,
    path: &Vec<Address>,
    bytes: &Option<Vec<BytesN<32>>>,
) -> Result<
    Vec<(Vec<Address>, BytesN<32>, Address)>, // (path, pool_hash, token_out)
    AdapterError,
> {
    // We check that bytes is not None
    let pool_hashes_vec = bytes.as_ref().ok_or(AdapterError::MissingPoolHashes)?;

    // path should have at least 2 elements. ifnot error WrongMinimumPathLength
    if path.len() < 2 {
        return Err(AdapterError::WrongMinimumPathLength);
    }
    // We check that the length of bytes is equal to the length of path - 1
    if pool_hashes_vec.len() != path.len().checked_sub(1).unwrap() {
        // unwrap safe as we checked the length of path
        return Err(AdapterError::WrongPoolHashesLength);
    }

    let mut swaps_chain = Vec::new(e);
    for i in 0..path.len() - 1 {
        let token_in = path.get(i).unwrap(); // This should be safe as we checked the length of path
        let token_out = path.get(i + 1).unwrap(); // This should be safe as we checked the length of path
        let pool_hash = pool_hashes_vec.get(i).unwrap(); // This should be safe as we checked the length of pool_hashes_vec

        let swap_chain_path = if token_in < token_out {
            vec![&e, token_in.clone(), token_out.clone()]
        } else {
            vec![&e, token_out.clone(), token_in.clone()]
        };

        swaps_chain.push_back((swap_chain_path, pool_hash.clone(), token_out.clone()));
    }

    Ok(swaps_chain)
}

pub fn protocol_swap_exact_tokens_for_tokens(
    e: &Env,
    amount_in: &i128,
    amount_out_min: &i128,
    path: &Vec<Address>, // (TokenA, TokenB, TokenC, TokenD), being TokenC the token to get
    to: &Address,
    bytes: &Option<Vec<BytesN<32>>>, // (pool_hash_0, pool_hash_1, pool_hash_2)
) -> Result<u128, AdapterError> {
    let aqua_router_address = crate::storage::get_protocol_address(&e);
    let aqua_router_client = AquariusRouterClient::new(&e, &aqua_router_address);
    let swaps_chain = convert_to_swaps_chain(e, path, bytes)?;

    let token_in = path.get(0).expect("Failed to get token in address"); // should be safe as we checked the length of path
    let token_out_address = path
        .get(path.len().checked_sub(1).unwrap())
        .expect("Failed to get token out address"); // should be safe as we checked the length of path

    let final_amount_out = aqua_router_client.swap_chained(
        &to,                        // user: Address
        &swaps_chain,               // swaps_chain: Vec<(Vec<Address>, BytesN<32>, Address)>,
        &token_in,                  // token_in: Address,
        &(*amount_in as u128),      // in_amount: i128,
        &(*amount_out_min as u128), // out_min: i128
    );

    Ok(final_amount_out)
}
