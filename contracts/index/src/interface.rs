use soroban_sdk::{ Address, Env };

use crate::stake::Stake;

pub trait IndexTrait {
    // fn get_total_shares(e: Env) -> u128;

    //
    fn mint(
        e: Env,
        user: Address,
        token: Address,
        amount: u128,
        destination: Option<Address>,
        max_slippage: Option<u64>
    );

    //
    fn redeem(e: Env, user: Address, share_amount: u128);
}

pub trait AdminInterface {
    //
    fn initialize(e: Env, admin: Address, token: Address);

    // Set unstaking period
    fn set_unstaking_period(e: Env, admin: Address, unstaking_period: u64);

    // Set max insurance
    fn set_max_shares(e: Env, admin: Address, max_shares: u128);

    fn rebalance(e: Env, admin: Address);

    // Stop staking instantly
    fn kill_mint(e: Env, admin: Address);
    fn kill_redeem(e: Env, admin: Address);
    fn kill_rebalance(e: Env, admin: Address);

    // Resume staking
    fn unkill_mint(e: Env, admin: Address);
    fn unkill_redeem(e: Env, admin: Address);
    fn unkill_rebalance(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_mint(e: Env) -> bool;
    fn get_is_killed_redeem(e: Env) -> bool;
    fn get_is_killed_rebalance(e: Env) -> bool;

    fn set_manager_address(e: Env, admin: Address, manager: Address);
    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address);
    fn distribute_manager_fees(e: Env, admin: Address);
    fn distribute_protocol_fees(e: Env, admin: Address);
    
    // Revenue Share Getters
    fn get_accumulated_manager_fees(e: Env) -> u128;
    fn get_accumulated_protocol_fees(e: Env) -> u128;
    fn get_manager_address(e: Env) -> Address;
    fn get_protocol_fee_recipient(e: Env) -> Address;
}
