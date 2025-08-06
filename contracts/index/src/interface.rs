use soroban_sdk::{ Address, Env };

use crate::stake::Stake;

pub trait IndexTrait {
    fn mint(
        e: Env,
        user: Address,
        token: Address,
        amount: u128,
        destination: Option<Address>,
        max_slippage: Option<u64>
    );

    fn redeem(e: Env, user: Address, share_amount: u128);


    fn get_token(e: Env) -> Address;

    fn get_factory(e: Env) -> Address;

    fn get_base_nav(e: Env) -> i128;

    fn get_initial_price(e: Env) -> i128;

    fn get_nav(e: Env) -> i128;

    fn get_price(e: Env) -> i128;

    fn get_total_shares(e: Env) -> u128;

    fn get_public_status(e: Env) -> bool;

    fn get_whitelist_status(e: Env, address: Address) -> bool;

    fn get_blacklist_status(e: Env, address: Address) -> bool;

    fn get_manager_fee_fraction(e: Env) -> u32;

    fn get_rebalance_threshold(e: Env) -> u64;

    fn get_last_rebalance_timestamp(e: Env) -> u64;

    fn get_last_updated_timestamp(e: Env) -> u64;

    fn get_total_mints(e: Env) -> u128;

    fn get_total_redemptions(e: Env) -> u128;

    fn get_total_fees(e: Env) -> u128;

    fn get_component(e: Env, token: Address) -> crate::storage::Component;

    fn get_component_balance(e: Env, token: Address) -> u128;

    fn get_last_fee_collection(e: Env) -> u64;
}

pub trait AdminInterface {
    fn initialize(e: Env, admin: Address, token: Address);

    fn rebalance(e: Env, admin: Address);

    fn distribute_manager_fees(e: Env, admin: Address);
    
    fn distribute_protocol_fees(e: Env, admin: Address);

    fn set_factory(e: Env, admin: Address, factory: Address);

    fn set_base_nav(e: Env, admin: Address, base_nav: i128);

    fn set_initial_price(e: Env, admin: Address, initial_price: i128);

    fn set_public_status(e: Env, admin: Address, public: bool);

    fn set_whitelist_status(e: Env, admin: Address, address: Address, status: bool);

    fn set_blacklist_status(e: Env, admin: Address, address: Address, status: bool);

    fn set_manager_address(e: Env, admin: Address, manager: Address);

    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address);

    fn set_manager_fee_fraction(e: Env, admin: Address, fee_fraction: u32);

    fn set_rebalance_threshold(e: Env, admin: Address, threshold: u64);

    fn set_unstaking_period(e: Env, admin: Address, unstaking_period: u64);

    fn set_max_shares(e: Env, admin: Address, max_shares: u128);

    //    _______     __       ____  ____   ________  _______  ________
    //   |   __ "\   /""\     ("  _||_ " | /"       )/"     "||"      "\
    //   (. |__) :) /    \    |   (  ) : |(:   \___/(: ______)(.  ___  :)
    //   |:  ____/ /' /\  \   (:  |  | . ) \___  \   \/    |  |: \   ) ||
    //   (|  /    //  __'  \   \\ \__/ //   __/  \\  // ___)_ (| (___\ ||
    //  /|__/ \  /   /  \\  \  /\\ __ //\  /" \   :)(:      "||:       :)
    // (_______)(___/    \___)(__________)(_______/  \_______)(________/

    fn kill_mint(e: Env, admin: Address);
    fn kill_redeem(e: Env, admin: Address);
    fn kill_rebalance(e: Env, admin: Address);

    fn unkill_mint(e: Env, admin: Address);
    fn unkill_redeem(e: Env, admin: Address);
    fn unkill_rebalance(e: Env, admin: Address);

    fn get_is_killed_mint(e: Env) -> bool;
    fn get_is_killed_redeem(e: Env) -> bool;
    fn get_is_killed_rebalance(e: Env) -> bool;

    fn get_accumulated_manager_fees(e: Env) -> u128;
    fn get_accumulated_protocol_fees(e: Env) -> u128;
}
