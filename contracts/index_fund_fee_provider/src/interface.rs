use soroban_sdk::{Address, Env, Map, Symbol, Vec};

pub trait IndexFundFeeProviderTrait {
    fn mint(
        e: Env,
        user: Address,
        index_fund: Address,
        token: Address,
        amount: u128,
        destination: Option<Address>,
    );

    fn redeem(e: Env, user: Address, index_fund: Address, index_token: Address, share_amount: u128);
}

pub trait AdminInterface {
    fn set_fee_token(e: Env, admin: Address, token: Address) -> Address;

    fn set_mint_fee(e: Env, admin: Address, fee: u128) -> u128;

    fn set_redeem_fee(e: Env, admin: Address, fee: u128) -> u128;

    fn get_fee_config(e: Env) -> (u128, u128);

    fn get_protocol_fees_by_token(e: Env, token: Address) -> u128;

    fn claim_protocol_fees(e: Env, admin: Address, token: Address, destination: Address) -> u128;

    // Set privileged addresses
    fn set_privileged_addrs(e: Env, admin: Address, fee_admin: Address, pause_admin: Address);

    // Get map of privileged roles
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>>;

    // Paused Ops

    // Stop fee instantly
    fn kill_fee(e: Env, admin: Address);

    // Resume fee
    fn unkill_fee(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_fee(e: Env) -> bool;
}
