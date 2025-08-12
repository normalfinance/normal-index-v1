use crate::errors::IndexError;
use crate::events::Events;
use crate::events::IndexEvents;
use crate::fees::{
    batch_collect_fees, collect_fees_before_action, force_collect_fees, get_effective_balance,
    get_last_batch_collection, get_users_with_accrued_fees, initialize_or_update_user_tracking,
    preview_accrued_fees, set_last_batch_collection,
};

use crate::index::vault_amount_to_shares;
use crate::interface::{
    AdminInterface, ComponentAction, ComponentAllocation, ComponentUpdate, IndexInfo, IndexMetrics,
    IndexStatus, IndexTrait, QueryInterface, RebalanceParams, RebalanceStatus,
};
use crate::storage::get_all_rebalance_authorities;
use crate::storage::get_blacklist_status;
use crate::storage::get_index_vault_amount;
use crate::storage::get_rebalance_authority_status;
use crate::storage::get_whitelist_status;
use crate::storage::remove_component;
use crate::storage::set_component;
use crate::storage::set_factory;
use crate::storage::set_last_rebalance_ts;
use crate::storage::set_last_updated_ts;
use crate::storage::set_manager_fee_fraction;
use crate::storage::set_public;
use crate::storage::set_rebalance_authority_status;
use crate::storage::set_total_mints;
use crate::storage::update_component_weight;
use crate::storage::{
    get_accumulated_manager_fees, get_accumulated_protocol_fees, get_fee_collection_enabled,
    get_manager_address, get_manager_fee_fraction, get_protocol_fee_recipient, get_total_fees,
    set_accumulated_manager_fees, set_accumulated_protocol_fees, set_fee_collection_enabled,
    set_last_fee_collection, set_manager_address, set_protocol_fee_recipient, set_total_fees,
};
use crate::storage::{
    get_all_component_balances, get_all_components, get_base_nav, get_component,
    get_component_balance, get_component_balance_safe, get_component_registry, get_factory,
    get_factory_safe, get_initial_price, get_is_killed_mint, get_is_killed_rebalance,
    get_is_killed_redeem, get_last_rebalance_ts, get_last_updated_ts, get_public,
    get_rebalance_threshold, get_total_mints, get_total_redemptions, set_is_killed_mint,
    set_is_killed_rebalance, set_is_killed_redeem, Component,
};
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::{MultipleAddressesManagementTrait, SingleAddressManagementTrait};
use access_control::role::Role;
use access_control::role::SymbolRepresentation;
use access_control::transfer::TransferOwnershipTrait;
use access_control::utils::{
    require_pause_admin_or_owner, require_pause_or_emergency_pause_admin_or_owner,
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};
use soroban_sdk::String;
use soroban_sdk::{
    contract, contractimpl, log, panic_with_error, token::TokenClient as SorobanTokenClient, vec,
    Address, BytesN, Env, IntoVal, Map, Symbol, Vec,
};
use token_share::get_token_share;
use token_share::get_total_shares;
use token_share::mint_shares;
use token_share::put_token_share;
use token_share::Client as ShareTokenClient;
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};
use utils::bump::bump_instance;
use utils::math::safe_math::SafeMath;
use utils::token::transfer_token;
use utils::token::validate_token_contracts;
use utils::validate;

#[contract]
pub struct Index;

impl Index {
    // __constructor
    // Initializes the ProviderSwapFeeCollector contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - factory: The address of the factory contract.
    //   - operator: The address authorized to claim funds.
    //   - fee_destination: The address where fees are sent.
    //   - max_swap_fee_fraction: The maximum fee in basis points (bps).
    pub fn __constructor(
        e: Env,
        admin: Address,
        factory: Address,
        name: String,
        token_symbol: String,
        description: String,
        public: bool,
        manager_fee_fraction: u32,
        initial_price: i128,
        initial_deposit: i128,
        components: Vec<Address>,
        rebalance_authorities: Vec<Address>, // New parameter for private index authorities
    ) {
        // set admin
        set_factory(&e, &factory);

        // deploy and initialize index token contract - placeholder implementation
        let share_contract = Address::from_str(&e, "placeholder");
        put_token_share(&e, share_contract);

        set_manager_fee_fraction(&e, &manager_fee_fraction);
        set_public(&e, &public);

        // Set the admin as the initial manager who will receive fees
        set_manager_address(&e, &admin);

        // Set up rebalance authorities for private indexes
        if !public {
            for authority in rebalance_authorities.iter() {
                set_rebalance_authority_status(&e, &authority, true);
            }
        }

        set_last_fee_collection(&e, &e.ledger().timestamp());
    }

    // Old fee collection function - DEPRECATED
    // Fee collection now handled by time-based system in fees.rs
}

// The `IndexTrait` trait provides the interface for interacting with a liquidity pool.
#[contractimpl]
impl IndexTrait for Index {
    fn mint(
        e: Env,
        user: Address,
        token: Address,
        amount: u128,
        destination: Option<Address>,
        max_slippage: Option<u64>,
    ) {
        user.require_auth();

        if get_is_killed_mint(&e) {
            panic_with_error!(e, IndexError::IndexMintKilled);
        }

        if get_blacklist_status(&e, &user) {
            panic_with_error!(e, IndexError::Blacklisted);
        }

        let is_public = get_public(&e);
        if !is_public {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&user, &Role::Admin);
            let is_whitelisted = get_whitelist_status(&e, &user);

            if !is_admin && !is_whitelisted {
                panic_with_error!(e, IndexError::NotWhitelisted);
            }
        }

        validate_token_contracts(&e, &vec![&e, token.clone()]);

        // ...

        let total_shares = get_total_shares(&e);

        let vault_amount = get_index_vault_amount(&e, &token);

        // validate!(
        //     &e,
        //     !(insurance_vault_amount == 0 && total_shares != 0),
        //     IndexError::InvalidIFForNewStakes
        // );

        let n_shares = vault_amount_to_shares(&e, amount, total_shares, vault_amount);

        // Collect any accrued fees before minting new shares
        let destination_user = match destination {
            Some(ref v) => v.clone(),
            None => user.clone(),
        };
        // Fee collection now handled at token level during mint_shares() call

        // Configure swaps
        let swaps_chain: Vec<(Vec<Address>, BytesN<32>, Address)> = Vec::new(&e);

        // Execute swaps
        // Deposit the token
        transfer_token(
            &e,
            &token,
            &user,
            &e.current_contract_address(),
            &(amount as i128),
        );
        if swaps_chain.len() == 0 {
            panic_with_error!(&e, IndexError::PathIsEmpty);
        }

        // execute swaps

        // Mint share tokens
        let value = match &destination {
            Some(v) => v.clone(),
            None => user.clone(),
        };
        mint_shares(&e, &value, n_shares as i128);

        // Initialize fee tracking for new user
        initialize_or_update_user_tracking(&e, &value, n_shares as i128);

        // Metrics
        set_total_mints(&e, &n_shares);

        // Emit enhanced mint event
        let current_time = e.ledger().timestamp();
        let nav_after = Self::get_nav(e.clone()) as u128;
        let total_shares_after = get_total_shares(&e);
        let share_price = Self::get_price(e.clone()) as u128;

        Events::new(&e).mint_executed(
            current_time,
            user.clone(),
            token,
            amount,
            n_shares,
            share_price,
            nav_after - amount, // Approximation of nav_before
            nav_after,
            total_shares,
            total_shares_after,
            0, // TODO: Calculate actual fees collected during mint
            destination,
        );

        // Also emit legacy event for backward compatibility
        Events::new(&e).mint(current_time, user);
    }

    fn redeem(e: Env, user: Address, share_amount: u128) {
        user.require_auth();

        if get_is_killed_redeem(&e) {
            panic_with_error!(e, IndexError::IndexRedeemKilled);
        }

        // Collect any accrued fees before redemption
        let (manager_fees, protocol_fees) =
            collect_fees_before_action(&e, &user, -(share_amount as i128));

        // TODO: Add actual redemption logic here
        // This would typically involve:
        // 1. Calculating redemption value
        // 2. Burning share tokens
        // 3. Transferring underlying assets back to user

        // For now, just emit event showing fees were collected
        if manager_fees > 0 || protocol_fees > 0 {
            // Fees already emitted in collect_fees_before_action
        }

        if get_blacklist_status(&e, &user) {
            panic_with_error!(e, IndexError::Blacklisted);
        }

        let is_public = get_public(&e);
        if !is_public {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&user, &Role::Admin);
            let is_whitelisted = get_whitelist_status(&e, &user);

            if !is_admin && !is_whitelisted {
                panic_with_error!(e, IndexError::NotWhitelisted);
            }
        }

        // Emit enhanced redemption event
        let current_time = e.ledger().timestamp();
        let nav_before = Self::get_nav(e.clone()) as u128;
        let total_shares_before = get_total_shares(&e);
        let share_price = Self::get_price(e.clone()) as u128;

        // TODO: Implement actual redemption logic to get accurate values
        let component_payouts = Map::new(&e); // Empty map for now

        Events::new(&e).redemption_executed(
            current_time,
            user.clone(),
            share_amount,
            share_price,
            nav_before,
            nav_before, // TODO: Calculate nav_after after redemption
            total_shares_before,
            total_shares_before - share_amount, // Approximation
            component_payouts,
            manager_fees + protocol_fees,
        );

        // Also emit legacy event for backward compatibility
        Events::new(&e).redeem(current_time, user);
    }

    fn get_token(e: Env) -> Address {
        get_token_share(&e)
    }

    fn get_factory(e: Env) -> Address {
        crate::storage::get_factory(&e)
    }

    fn get_base_nav(e: Env) -> u128 {
        crate::storage::get_base_nav(&e)
    }

    fn get_initial_price(e: Env) -> u128 {
        crate::storage::get_initial_price(&e)
    }

    fn get_nav(e: Env) -> i128 {
        let base_nav = crate::storage::get_base_nav(&e) as i128;
        let total_shares = get_total_shares(&e);
        if total_shares == 0 {
            return base_nav;
        }

        let token = get_token_share(&e);
        let vault_amount = crate::storage::get_index_vault_amount(&e, &token) as i128;
        vault_amount
    }

    fn get_price(e: Env) -> i128 {
        let nav = Self::get_nav(e.clone());
        let total_shares = get_total_shares(&e);
        if total_shares == 0 {
            return crate::storage::get_initial_price(&e) as i128;
        }
        nav / (total_shares as i128)
    }

    fn get_total_shares(e: Env) -> u128 {
        get_total_shares(&e)
    }

    fn get_public_status(e: Env) -> bool {
        crate::storage::get_public(&e)
    }

    fn get_whitelist_status(e: Env, address: Address) -> bool {
        crate::storage::get_whitelist_status(&e, &address)
    }

    fn get_blacklist_status(e: Env, address: Address) -> bool {
        crate::storage::get_blacklist_status(&e, &address)
    }

    fn get_manager_fee_fraction(e: Env) -> u32 {
        crate::storage::get_manager_fee_fraction(&e)
    }

    fn get_fee_collection_enabled(e: Env) -> bool {
        crate::storage::get_fee_collection_enabled(&e)
    }

    fn get_rebalance_threshold(e: Env) -> u64 {
        crate::storage::get_rebalance_threshold(&e)
    }

    fn get_last_rebalance_timestamp(e: Env) -> u64 {
        crate::storage::get_last_rebalance_ts(&e)
    }

    fn get_last_updated_timestamp(e: Env) -> u64 {
        crate::storage::get_last_updated_ts(&e)
    }

    fn get_total_mints(e: Env) -> u128 {
        crate::storage::get_total_mints(&e)
    }

    fn get_total_redemptions(e: Env) -> u128 {
        crate::storage::get_total_redemptions(&e)
    }

    fn get_total_fees(e: Env) -> u128 {
        crate::storage::get_total_fees(&e)
    }

    fn get_component(e: Env, token: Address) -> crate::storage::Component {
        crate::storage::get_component(&e, token)
    }

    fn get_component_balance(e: Env, token: Address) -> u128 {
        crate::storage::get_component_balance(&e, token)
    }

    fn get_last_fee_collection(e: Env) -> u64 {
        crate::storage::get_last_fee_collection(&e)
    }

    /// Transfer shares between users with proper fee handling
    fn transfer_shares(e: Env, from: Address, to: Address, amount: u128) {
        from.require_auth();

        // Collect fees from both sender and receiver before transfer
        collect_fees_before_action(&e, &from, -(amount as i128));
        collect_fees_before_action(&e, &to, amount as i128);

        // Execute the token transfer
        let share_token = get_token_share(&e);
        SorobanTokenClient::new(&e, &share_token).transfer(&from, &to, &(amount as i128));

        // Update fee tracking for both users
        initialize_or_update_user_tracking(&e, &from, -(amount as i128));
        initialize_or_update_user_tracking(&e, &to, amount as i128);
    }

    /// Transfer shares from allowance with proper fee handling  
    fn transfer_shares_from(e: Env, spender: Address, from: Address, to: Address, amount: u128) {
        spender.require_auth();

        // Collect fees from both sender and receiver before transfer
        collect_fees_before_action(&e, &from, -(amount as i128));
        collect_fees_before_action(&e, &to, amount as i128);

        // Execute the token transfer from allowance
        let share_token = get_token_share(&e);
        SorobanTokenClient::new(&e, &share_token).transfer_from(
            &spender,
            &from,
            &to,
            &(amount as i128),
        );

        // Update fee tracking for both users
        initialize_or_update_user_tracking(&e, &from, -(amount as i128));
        initialize_or_update_user_tracking(&e, &to, amount as i128);
    }
}

// The `UpgradeableContract` trait provides the interface for upgrading the contract.
#[contractimpl]
impl UpgradeableContract for Index {
    // Returns the version of the contract.
    //
    // # Returns
    //
    // The version of the contract as a u32.
    fn version() -> u32 {
        150
    }

    // Commits a new wasm hash for a future upgrade.
    // The upgrade will be available through `apply_upgrade` after the standard upgrade delay
    // unless the system is in emergency mode.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `new_wasm_hash` - The new wasm hash to commit.
    fn commit_upgrade(e: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        commit_upgrade(&e, &new_wasm_hash);
        UpgradeEvents::new(&e).commit_upgrade(Vec::from_array(&e, [new_wasm_hash.clone()]));
    }

    // Applies the committed upgrade.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn apply_upgrade(e: Env, admin: Address) -> BytesN<32> {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        let new_wasm_hash = apply_upgrade(&e);
        UpgradeEvents::new(&e).apply_upgrade(Vec::from_array(&e, [new_wasm_hash.clone()]));
        new_wasm_hash
    }

    // Reverts the committed upgrade.
    // This can be used to cancel a previously committed upgrade.
    // The upgrade will be canceled only if it has not been applied yet.
    // If the upgrade has already been applied, it cannot be reverted.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn revert_upgrade(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        revert_upgrade(&e);
        UpgradeEvents::new(&e).revert_upgrade();
    }

    // Sets the emergency mode.
    // When the emergency mode is set to true, the contract will allow instant upgrades without the delay.
    // This is useful in case of critical issues that need to be fixed immediately.
    // When the emergency mode is set to false, the contract will require the standard upgrade delay.
    // The emergency mode can only be set by the emergency admin.
    //
    // # Arguments
    //
    // * `emergency_admin` - The address of the emergency admin.
    // * `value` - The value to set the emergency mode to.
    fn set_emergency_mode(e: Env, emergency_admin: Address, value: bool) {
        emergency_admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&emergency_admin, &Role::EmergencyAdmin);
        set_emergency_mode(&e, &value);
        AccessControlEvents::new(&e).set_emergency_mode(value);
    }

    // Returns the emergency mode flag value.
    fn get_emergency_mode(e: Env) -> bool {
        get_emergency_mode(&e)
    }
}

// The `AdminInterface` trait provides the interface for administrative actions.
#[contractimpl]
impl AdminInterface for Index {
    // Initializes the admin user.
    //
    // # Arguments
    //
    // * `account` - The address of the admin user.
    fn initialize(e: Env, admin: Address, token: Address) {
        admin.require_auth();

        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, AccessControlError::AdminAlreadySet);
        }
        access_control.set_role_address(&Role::Admin, &admin);

        put_token_share(&e, token);

        // No complex registration needed!
        // The token's admin IS this index contract, so fee calls work automatically
    }

    fn rebalance(e: Env, caller: Address, params: RebalanceParams) {
        caller.require_auth();

        if get_is_killed_rebalance(&e) {
            panic_with_error!(e, IndexError::IndexRebalanceKilled);
        }

        if get_blacklist_status(&e, &caller) {
            panic_with_error!(e, IndexError::Blacklisted);
        }

        let is_public = get_public(&e);
        if !is_public {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&caller, &Role::Admin);
            let is_whitelisted = get_whitelist_status(&e, &caller);

            if !is_admin && !is_whitelisted {
                panic_with_error!(e, IndexError::NotWhitelisted);
            }
        }

        // Check rebalance threshold timing
        let current_time = e.ledger().timestamp();
        let last_rebalance = get_last_rebalance_ts(&e);
        let threshold = get_rebalance_threshold(&e);

        if current_time < last_rebalance + threshold {
            panic_with_error!(e, IndexError::RebalanceTooSoon);
        }

        // Permission checks based on index type
        if is_public {
            // Public index: requires DAO proposal approval (for now, only admin)
            Index::validate_public_rebalance(&e, &caller, &params);
        } else {
            // Private index: admin or rebalance authority
            Index::validate_private_rebalance(&e, &caller);
        }

        // Capture pre-rebalancing state
        let nav_before = Self::get_nav(e.clone()) as u128;
        let components_before = get_all_components(&e);

        // Execute rebalancing logic
        Index::execute_rebalancing(&e, caller.clone(), params.clone());

        // Capture post-rebalancing state
        let nav_after = Self::get_nav(e.clone()) as u128;
        let components_after = get_all_components(&e);

        // Update timestamps
        set_last_rebalance_ts(&e, &current_time);
        set_last_updated_ts(&e, &current_time);

        // Emit enhanced rebalancing event
        Events::new(&e).rebalance_executed(
            current_time,
            caller.clone(),
            nav_before,
            nav_after,
            components_before,
            components_after,
            params.component_updates.len() as u32,
            0,                                          // TODO: Calculate actual gas cost
            (nav_after as i128) - (nav_before as i128), // Performance impact
        );

        // Also emit legacy event for backward compatibility
        Events::new(&e).rebalance(current_time, caller);
    }

    fn set_rebalance_authority(e: Env, admin: Address, authority: Address, status: bool) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        // Get old status before updating
        let old_status = get_rebalance_authority_status(&e, &authority);
        set_rebalance_authority_status(&e, &authority, status);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).rebalance_authority_updated_detailed(
            current_time,
            admin.clone(),
            authority.clone(),
            old_status,
            status,
        );

        // Also emit legacy event for backward compatibility
        Events::new(&e).rebalance_authority_updated(authority, status);
    }

    // Stops index mints instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_mint(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_mint(&e, &true);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).operation_killed(current_time, admin.clone(), Symbol::new(&e, "mint"));
        // Also emit legacy event for backward compatibility
        Events::new(&e).kill_deposit();
    }

    // Stops index redemptions instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_redeem(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_redeem(&e, &true);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).operation_killed(current_time, admin.clone(), Symbol::new(&e, "redeem"));
        // Also emit legacy event for backward compatibility
        Events::new(&e).kill_request_withdraw();
    }

    // Stops the pool swaps instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_rebalance(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_rebalance(&e, &true);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).operation_killed(current_time, admin.clone(), Symbol::new(&e, "rebalance"));
        // Also emit legacy event for backward compatibility
        Events::new(&e).kill_withdraw();
    }

    // Resumes the pool deposits.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_mint(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_mint(&e, &false);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).operation_unkilled(current_time, admin.clone(), Symbol::new(&e, "mint"));
        // Also emit legacy event for backward compatibility
        Events::new(&e).unkill_deposit();
    }

    // Resumes the pool swaps.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_redeem(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_redeem(&e, &false);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).operation_unkilled(current_time, admin.clone(), Symbol::new(&e, "redeem"));
        // Also emit legacy event for backward compatibility
        Events::new(&e).unkill_request_withdraw();
    }

    // Resumes the pool withdrawals.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_rebalance(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_rebalance(&e, &false);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).operation_unkilled(
            current_time,
            admin.clone(),
            Symbol::new(&e, "rebalance"),
        );
        // Also emit legacy event for backward compatibility
        Events::new(&e).unkill_withdraw();
    }

    // Get deposit killswitch status.
    fn get_is_killed_mint(e: Env) -> bool {
        get_is_killed_mint(&e)
    }

    // Get swap killswitch status.
    fn get_is_killed_redeem(e: Env) -> bool {
        get_is_killed_redeem(&e)
    }

    // Get withdraw killswitch status.
    fn get_is_killed_rebalance(e: Env) -> bool {
        get_is_killed_rebalance(&e)
    }

    fn set_manager_address(e: Env, admin: Address, manager: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_manager = get_manager_address(&e);
        set_manager_address(&e, &manager);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).manager_address_updated(
            current_time,
            admin.clone(),
            old_manager.clone(),
            manager.clone(),
        );
        // Also emit legacy event for backward compatibility
        Events::new(&e).manager_address_updated_legacy(old_manager, manager);
    }

    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_recipient = get_protocol_fee_recipient(&e);
        set_protocol_fee_recipient(&e, &recipient);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).protocol_fee_recipient_updated(
            current_time,
            admin.clone(),
            old_recipient.clone(),
            recipient.clone(),
        );
        // Also emit legacy event for backward compatibility
        Events::new(&e).protocol_fee_recipient_updated_legacy(old_recipient, recipient);
    }

    fn distribute_manager_fees(e: Env, admin: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let accumulated_fees = get_accumulated_manager_fees(&e);
        if accumulated_fees == 0 {
            return;
        }

        let manager = get_manager_address(&e);
        if manager == Address::from_str(&e, "") {
            panic_with_error!(&e, IndexError::ManagerNotSet);
        }

        set_accumulated_manager_fees(&e, &0);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).fees_distributed_to_manager(
            current_time,
            manager.clone(),
            accumulated_fees,
            accumulated_fees, // total_accumulated_before
            0,                // total_accumulated_after (reset to 0)
        );

        // Also emit legacy event for backward compatibility
        Events::new(&e).manager_fees_distributed(manager.clone(), accumulated_fees);

        //This is a placeholder for the manager to claim their fees
    }

    fn distribute_protocol_fees(e: Env, admin: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let accumulated_fees = get_accumulated_protocol_fees(&e);
        if accumulated_fees == 0 {
            return;
        }

        let protocol_recipient = get_protocol_fee_recipient(&e);
        if protocol_recipient == Address::from_str(&e, "") {
            panic_with_error!(&e, IndexError::ProtocolRecipientNotSet);
        }

        set_accumulated_protocol_fees(&e, &0);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).fees_distributed_to_protocol(
            current_time,
            protocol_recipient.clone(),
            accumulated_fees,
            accumulated_fees, // total_accumulated_before
            0,                // total_accumulated_after (reset to 0)
        );

        // Also emit legacy event for backward compatibility
        Events::new(&e).protocol_fees_distributed(protocol_recipient.clone(), accumulated_fees);

        //This is a placeholder for the protocol to claim their fees
    }

    fn get_accumulated_manager_fees(e: Env) -> u128 {
        get_accumulated_manager_fees(&e)
    }

    fn get_accumulated_protocol_fees(e: Env) -> u128 {
        get_accumulated_protocol_fees(&e)
    }

    fn set_factory(e: Env, admin: Address, factory: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_factory(&e, &factory);
    }

    fn set_base_nav(e: Env, admin: Address, base_nav: u128) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_nav = get_base_nav(&e);
        crate::storage::set_base_nav(&e, &base_nav);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).base_nav_updated(current_time, admin, old_nav, base_nav);
    }

    fn set_initial_price(e: Env, admin: Address, initial_price: u128) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_price = get_initial_price(&e);
        crate::storage::set_initial_price(&e, &initial_price);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).initial_price_updated(current_time, admin, old_price, initial_price);
    }

    fn set_public_status(e: Env, admin: Address, public: bool) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_status = get_public(&e);
        crate::storage::set_public(&e, &public);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).public_status_updated(current_time, admin, old_status, public);
    }

    fn set_whitelist_status(e: Env, admin: Address, address: Address, status: bool) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_status = get_whitelist_status(&e, &address);
        crate::storage::set_whitelist_status(&e, &address, status);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).whitelist_status_updated(current_time, admin, address, old_status, status);
    }

    fn set_blacklist_status(e: Env, admin: Address, address: Address, status: bool) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_status = get_blacklist_status(&e, &address);
        crate::storage::set_blacklist_status(&e, &address, status);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).blacklist_status_updated(current_time, admin, address, old_status, status);
    }

    fn set_manager_fee_fraction(e: Env, admin: Address, fee_fraction: u32) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_fee_fraction = get_manager_fee_fraction(&e);
        crate::storage::set_manager_fee_fraction(&e, &fee_fraction);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).manager_fee_fraction_updated(
            current_time,
            admin,
            old_fee_fraction,
            fee_fraction,
        );
    }

    fn set_fee_collection_enabled(e: Env, admin: Address, enabled: bool) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_status = get_fee_collection_enabled(&e);
        set_fee_collection_enabled(&e, &enabled);

        // Emit event if status changed
        if old_status != enabled {
            Events::new(&e).fee_collection_toggled(enabled);
        }
    }

    fn set_rebalance_threshold(e: Env, admin: Address, threshold: u64) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_threshold = get_rebalance_threshold(&e);
        crate::storage::set_rebalance_threshold(&e, &threshold);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).rebalance_threshold_updated(current_time, admin, old_threshold, threshold);
    }
}

// The `TransferableContract` trait provides the interface for transferring ownership of the contract.
#[contractimpl]
impl TransferableContract for Index {
    // Commits an ownership transfer.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `role_name` - The name of the role to transfer ownership of. The role must be one of the following:
    //     * `Admin`
    //     * `EmergencyAdmin`
    // * `new_address` - New address for the role
    fn commit_transfer_ownership(e: Env, admin: Address, role_name: Symbol, new_address: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        access_control.commit_transfer_ownership(&role, &new_address);
        AccessControlEvents::new(&e).commit_transfer_ownership(role, new_address);
    }

    // Applies the committed ownership transfer.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `role_name` - The name of the role to transfer ownership of. The role must be one of the following:
    //     * `Admin`
    //     * `EmergencyAdmin`
    fn apply_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        let new_address = access_control.apply_transfer_ownership(&role);
        AccessControlEvents::new(&e).apply_transfer_ownership(role, new_address);
    }

    // Reverts the committed ownership transfer.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `role_name` - The name of the role to transfer ownership of. The role must be one of the following:
    //     * `Admin`
    //     * `EmergencyAdmin`
    fn revert_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        access_control.revert_transfer_ownership(&role);
        AccessControlEvents::new(&e).revert_transfer_ownership(role);
    }

    // Returns the future address for the role.
    // The future address is the address that the ownership of the role will be transferred to.
    // The future address is set using the `commit_transfer_ownership` function.
    // The address will be defaulted to the current address if the transfer is not committed.
    //
    // # Arguments
    //
    // * `role_name` - The name of the role to get the future address for. The role must be one of the following:
    //    * `Admin`
    //    * `EmergencyAdmin`
    fn get_future_address(e: Env, role_name: Symbol) -> Address {
        let access_control = AccessControl::new(&e);
        let role = Role::from_symbol(&e, role_name);
        match access_control.get_transfer_ownership_deadline(&role) {
            0 => match access_control.get_role_safe(&role) {
                Some(address) => address,
                None => panic_with_error!(&e, AccessControlError::RoleNotFound),
            },
            _ => access_control.get_future_address(&role),
        }
    }
}

// Implementation of QueryInterface trait for Index contract
#[contractimpl]
impl QueryInterface for Index {
    // Comprehensive index information
    fn get_index_info(e: Env) -> IndexInfo {
        IndexInfo {
            address: e.current_contract_address(),
            token_address: get_token_share(&e),
            total_shares: get_total_shares(&e),
            base_nav: get_base_nav(&e),
            initial_price: get_initial_price(&e),
            is_public: get_public(&e),
            manager_fee_fraction: get_manager_fee_fraction(&e),
            manager_address: get_manager_address(&e),
            protocol_fee_recipient: get_protocol_fee_recipient(&e),
            accumulated_manager_fees: get_accumulated_manager_fees(&e),
            accumulated_protocol_fees: get_accumulated_protocol_fees(&e),
            last_rebalance_ts: get_last_rebalance_ts(&e),
            last_updated_ts: get_last_updated_ts(&e),
            total_mints: get_total_mints(&e),
            total_redemptions: get_total_redemptions(&e),
            total_fees: get_total_fees(&e),
            is_killed_mint: get_is_killed_mint(&e),
            is_killed_redeem: get_is_killed_redeem(&e),
            is_killed_rebalance: get_is_killed_rebalance(&e),
        }
    }

    // Component and balance queries
    fn get_all_components(e: Env) -> Map<Address, Component> {
        get_all_components(&e)
    }

    fn get_component_info(e: Env, token: Address) -> Component {
        get_component(&e, token)
    }

    fn get_all_component_balances(e: Env) -> Map<Address, u128> {
        get_all_component_balances(&e)
    }

    fn get_total_index_value(e: Env) -> u128 {
        let mut total_value: u128 = 0;

        // Get all component addresses from registry
        let component_addresses = get_component_registry(&e);

        // Iterate through each component to calculate total portfolio value
        for component_address in component_addresses.iter() {
            // Get the component balance (how much of this token the index holds)
            let balance = match get_component_balance_safe(&e, component_address.clone()) {
                Some(bal) => bal,
                None => 0u128, // If no balance stored, treat as 0
            };

            if balance > 0 {
                // Get the token price - for now we'll use a placeholder approach
                let token_price =
                    Index::get_token_price_in_base_currency(&e, component_address.clone());

                // Calculate value: balance * price
                let component_value = balance.saturating_mul(token_price);
                total_value = total_value.saturating_add(component_value);
            }
        }

        total_value
    }
    // Financial metrics
    fn get_index_metrics(e: Env) -> IndexMetrics {
        let current_nav = Index::get_current_nav(e.clone());
        let share_price = Index::get_share_price(e.clone());

        IndexMetrics {
            total_shares: get_total_shares(&e),
            total_mints: get_total_mints(&e),
            total_redemptions: get_total_redemptions(&e),
            total_fees: get_total_fees(&e),
            accumulated_manager_fees: get_accumulated_manager_fees(&e),
            accumulated_protocol_fees: get_accumulated_protocol_fees(&e),
            current_nav,
            share_price,
        }
    }

    fn get_share_price(e: Env) -> u128 {
        let total_shares = get_total_shares(&e);
        if total_shares == 0 {
            let ip = get_initial_price(&e);
            return if ip < 0 { 0 } else { ip as u128 };
        }

        let total_value = Index::get_total_index_value(e.clone());
        if total_value == 0 {
            let ip = get_initial_price(&e);
            return if ip < 0 { 0 } else { ip as u128 };
        }

        // Share price = Total Portfolio Value / Total Shares
        total_value / total_shares
    }

    fn get_current_nav(e: Env) -> u128 {
        // NAV (Net Asset Value) is the total value of all holdings
        Index::get_total_index_value(e)
    }
    //  get_is_killed_rebalance

    // Operational status
    fn get_index_status(e: Env) -> IndexStatus {
        let current_time = e.ledger().timestamp();
        let last_rebalance = get_last_rebalance_ts(&e);
        let threshold = get_rebalance_threshold(&e);
        let can_rebalance =
            (current_time >= last_rebalance + threshold) && !get_is_killed_rebalance(&e);

        IndexStatus {
            is_killed_mint: get_is_killed_mint(&e),
            is_killed_redeem: get_is_killed_redeem(&e),
            is_killed_rebalance: get_is_killed_rebalance(&e),
            is_public: get_public(&e),
            can_rebalance,
            last_rebalance_ts: get_last_rebalance_ts(&e),
            rebalance_threshold: get_rebalance_threshold(&e),
        }
    }

    fn can_rebalance(e: Env) -> bool {
        if get_is_killed_rebalance(&e) {
            return false;
        }

        let current_time = e.ledger().timestamp();
        let last_rebalance = get_last_rebalance_ts(&e);
        let threshold = get_rebalance_threshold(&e);

        current_time >= last_rebalance + threshold
    }

    // Rebalancing queries
    fn get_rebalance_status(e: Env) -> RebalanceStatus {
        let current_time = e.ledger().timestamp();
        let last_rebalance = get_last_rebalance_ts(&e);
        let threshold = get_rebalance_threshold(&e);
        let can_rebalance =
            (current_time >= last_rebalance + threshold) && !get_is_killed_rebalance(&e);
        let time_until_next = if can_rebalance {
            0
        } else {
            (last_rebalance + threshold) - current_time
        };

        // Get authorized rebalancers for private indexes
        let authorized_rebalancers = if get_public(&e) {
            Vec::new(&e) // Public indexes don't have individual authorities
        } else {
            get_all_rebalance_authorities(&e)
        };

        RebalanceStatus {
            can_rebalance,
            time_until_next_rebalance: time_until_next,
            last_rebalance_ts: last_rebalance,
            rebalance_threshold: threshold,
            is_public: get_public(&e),
            authorized_rebalancers,
        }
    }

    fn can_address_rebalance(e: Env, caller: Address) -> bool {
        if get_is_killed_rebalance(&e) {
            return false;
        }

        let current_time = e.ledger().timestamp();
        let last_rebalance = get_last_rebalance_ts(&e);
        let threshold = get_rebalance_threshold(&e);

        if current_time < last_rebalance + threshold {
            return false;
        }

        let access_control = AccessControl::new(&e);
        let is_public = get_public(&e);

        if is_public {
            // Public index: only admin for now (later DAO)
            access_control.address_has_role(&caller, &Role::Admin)
        } else {
            // Private index: admin or rebalance authority
            access_control.address_has_role(&caller, &Role::Admin)
                || get_rebalance_authority_status(&e, &caller)
        }
    }

    fn get_component_allocation(e: Env) -> Map<Address, ComponentAllocation> {
        let mut allocations = Map::new(&e);
        let components = get_all_components(&e);
        let current_nav = Index::get_current_nav(e.clone());

        for (token, component) in components.iter() {
            let current_balance = get_component_balance_safe(&e, token.clone()).unwrap_or(0);
            let target_balance = if current_nav > 0 {
                (current_nav * component.weight as u128) / 10000
            } else {
                0
            };
            let percentage = if current_nav > 0 {
                (current_balance * 10000) / current_nav
            } else {
                0
            };

            let allocation = ComponentAllocation {
                component: component.clone(),
                current_balance,
                target_balance,
                percentage_of_nav: percentage,
            };

            allocations.set(token, allocation);
        }

        allocations
    }

    fn get_rebalance_authorities(e: Env) -> Vec<Address> {
        get_all_rebalance_authorities(&e)
    }
}

// Additional helper functions for Index
impl Index {
    // Helper function to get token price in base currency
    // This is where oracle integration would happen
    pub fn get_token_price_in_base_currency(e: &Env, token: Address) -> u128 {
        // IMPLEMENTATION STRATEGY:
        // 1. Try to get price from factory's swap aggregator
        // 2. Fall back to stored price ratios
        // 3. Default to 1:1 ratio if no price available

        // Attempt to get factory address for price discovery
        match get_factory_safe(&e) {
            Some(factory_address) => {
                // Try to get a realistic price using the factory's aggregator
                match Index::get_price_via_factory_aggregator(&e, &factory_address, &token) {
                    Some(price) => price,
                    None => {
                        // Fall back to component weight-based pricing
                        Index::get_price_from_component_weight(&e, &token)
                    }
                }
            }
            None => {
                // No factory connection, use component weight-based pricing
                Index::get_price_from_component_weight(&e, &token)
            }
        }
    }

    // Helper function to get price via factory aggregator (simulation)
    fn get_price_via_factory_aggregator(
        e: &Env,
        _factory_address: &Address,
        token: &Address,
    ) -> Option<u128> {
        // Placeholder: aggregator not implemented yet. Return None to fall back to weight-based pricing.
        let _ = (e, token);
        None
    }

    // Helper function to get price based on component weight
    fn get_price_from_component_weight(e: &Env, token: &Address) -> u128 {
        // Get component information to use weight as a price indicator
        match crate::storage::get_component_safe(e, token.clone()) {
            Some(component) => {
                // Use component weight as a price multiplier
                // Higher weight = more valuable component
                // Weight is typically in basis points (e.g., 5000 = 50%)
                let base_price = 1_000_000u128; // Base price of 1.0 (6 decimals)
                let weight_multiplier = if component.weight > 0 {
                    // Scale weight (basis points) to a reasonable price multiplier
                    // Weight 10000 (100%) = 1.0x, Weight 5000 (50%) = 0.5x, etc.
                    component.weight.max(1000) // Minimum 10% weight
                } else {
                    1000u128 // Default 10% weight
                };

                // Calculate price: base_price * (weight / 10000)
                base_price.saturating_mul(weight_multiplier) / 10000
            }
            None => {
                // Token not found in components, use default price
                1_000_000u128 // 1.0 with 6 decimals
            }
        }
    }

    // Rebalancing helper functions
    fn validate_private_rebalance(e: &Env, caller: &Address) {
        let access_control = AccessControl::new(e);

        // Allow admin or rebalance authority
        if !access_control.address_has_role(caller, &Role::Admin)
            && !get_rebalance_authority_status(e, caller)
        {
            panic_with_error!(e, IndexError::UnauthorizedRebalance);
        }
    }

    fn validate_public_rebalance(e: &Env, caller: &Address, _params: &RebalanceParams) {
        // For now, only admin can rebalance public indexes
        // Later, add DAO proposal validation logic
        let access_control = AccessControl::new(e);
        if !access_control.address_has_role(caller, &Role::Admin) {
            panic_with_error!(e, IndexError::PublicRebalanceRequiresProposal);
        }

        // TODO: Validate DAO proposal approval
        // if let Some(proposal_id) = params.proposal_id {
        //     validate_dao_proposal_approval(e, proposal_id);
        // }
    }

    fn execute_rebalancing(e: &Env, admin: Address, params: RebalanceParams) {
        let start_time = e.ledger().timestamp();
        let nav_before = Self::get_nav(e.clone()) as u128;

        let mut total_weight = 0u128;
        let mut components_updated = 0u32;

        // Validate and execute component updates
        for update in params.component_updates.iter() {
            match update.action {
                ComponentAction::Add => {
                    // Create component with symbol (simplified for now)
                    let component = Component {
                        asset: Symbol::new(e, "TOKEN"), // Simplified - would need proper token symbol
                        weight: update.new_weight,
                    };
                    set_component(e, update.token.clone(), component);
                    total_weight += update.new_weight;
                    components_updated += 1;

                    // Get component balance for NAV impact calculation
                    let initial_balance =
                        get_component_balance_safe(e, update.token.clone()).unwrap_or(0);
                    let current_time = e.ledger().timestamp();

                    // Emit enhanced event
                    Events::new(e).component_added_detailed(
                        current_time,
                        admin.clone(),
                        update.token.clone(),
                        update.new_weight,
                        initial_balance,
                        0, // TODO: Calculate actual NAV impact
                    );

                    // Also emit legacy event for backward compatibility
                    Events::new(e).component_added(update.token.clone(), update.new_weight);
                }
                ComponentAction::Remove => {
                    // Get component info before removing
                    let component = get_component(e, update.token.clone()); // This will panic if not found
                    let final_balance =
                        get_component_balance_safe(e, update.token.clone()).unwrap_or(0);
                    let current_time = e.ledger().timestamp();

                    remove_component(e, update.token.clone());
                    components_updated += 1;

                    // Emit enhanced event
                    Events::new(e).component_removed_detailed(
                        current_time,
                        admin.clone(),
                        update.token.clone(),
                        final_balance,
                        final_balance, // proceeds_distributed (approximation)
                        0,             // TODO: Calculate actual NAV impact
                    );

                    // Also emit legacy event for backward compatibility
                    Events::new(e).component_removed(update.token.clone());
                }
                ComponentAction::UpdateWeight => {
                    // Get component info before and after updating
                    let old_component = get_component(e, update.token.clone()); // This will panic if not found
                    let old_weight = old_component.weight;
                    let balance_before =
                        get_component_balance_safe(e, update.token.clone()).unwrap_or(0);
                    let current_time = e.ledger().timestamp();

                    update_component_weight(e, update.token.clone(), update.new_weight);
                    total_weight += update.new_weight;
                    components_updated += 1;

                    let balance_after =
                        get_component_balance_safe(e, update.token.clone()).unwrap_or(0);

                    // Emit enhanced event
                    Events::new(e).component_weight_updated_detailed(
                        current_time,
                        admin.clone(),
                        update.token.clone(),
                        old_weight,
                        update.new_weight,
                        balance_before,
                        balance_after,
                        0, // TODO: Calculate actual NAV impact
                    );

                    // Also emit legacy event for backward compatibility
                    Events::new(e).component_weight_updated(
                        update.token.clone(),
                        old_weight,
                        update.new_weight,
                    );
                }
            }
        }

        // Validate total weights equal 100% (10000 basis points) for add/update operations
        let has_weight_operations = params.component_updates.iter().any(|u| {
            matches!(
                u.action,
                ComponentAction::Add | ComponentAction::UpdateWeight
            )
        });

        if has_weight_operations && total_weight != 10000 {
            panic_with_error!(e, IndexError::InvalidWeightSum);
        }

        // Generate and execute swap transactions to reach target allocation
        let swaps = crate::index::generate_rebalance_swaps(e, &params);
        let total_swaps = swaps.len() as u32;

        if total_swaps > 0 {
            let _swap_results = crate::index::execute_swaps(e, swaps);
        }

        // Capture end state for enhanced event
        let end_time = e.ledger().timestamp();
        let nav_after = Self::get_nav(e.clone()) as u128;
        let duration_ms = (end_time - start_time) * 1000; // Convert to milliseconds
        let performance_delta = (nav_after as i128) - (nav_before as i128);

        // Emit enhanced completion event
        Events::new(e).rebalance_completed_detailed(
            end_time,
            admin,
            components_updated,
            total_swaps,
            0, // TODO: Calculate actual total gas cost
            performance_delta,
            nav_before,
            nav_after,
            duration_ms,
        );

        // Also emit legacy event for backward compatibility
        Events::new(e).rebalance_completed(
            e.current_contract_address(),
            components_updated,
            total_swaps,
        );
    }
}

// Fee system management functions

#[contractimpl]
impl Index {
    /// Preview accrued fees for a user without collecting them
    pub fn preview_fees(e: Env, user: Address) -> (u128, u128) {
        preview_accrued_fees(&e, &user)
    }

    /// Get effective balance (balance minus accrued fees)
    pub fn get_effective_balance(e: Env, user: Address) -> i128 {
        get_effective_balance(&e, &user)
    }

    /// Manually trigger fee collection for a user (admin function)
    pub fn collect_fees(e: Env, admin: Address, user: Address) -> (u128, u128) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        collect_fees_before_action(&e, &user, 0) // 0 balance change for manual collection
    }

    // Note: Minimum fee threshold is now set universally by the Factory contract
    // and cannot be changed after deployment. This ensures protocol consistency.

    /// Batch collect fees from multiple users (admin function for periodic collection)
    pub fn batch_collect_fees(e: Env, admin: Address, users: Vec<Address>) -> (u128, u128) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let result = batch_collect_fees(&e, users);

        // Update last batch collection timestamp
        set_last_batch_collection(&e, e.ledger().timestamp());

        result
    }

    /// Called by token contract to enforce fee collection for transfers and burns
    /// This ensures fees are collected regardless of where tokens are traded (external DEXes)
    pub fn collect_fees_before_operation(
        e: Env,
        from: Address,
        amount: i128,
        to: Option<Address>, // Some(address) for transfers, None for burns
    ) -> (u128, u128) {
        // No auth required - this is called by the trusted token contract

        // Collect fees from sender before they transfer/burn tokens
        let (manager_fees, protocol_fees) = collect_fees_before_action(&e, &from, -(amount));

        // Update tracking based on operation type
        match to {
            Some(recipient) => {
                // Transfer: update tracking for both users
                initialize_or_update_user_tracking(&e, &from, -(amount)); // Sender: reduce balance
                initialize_or_update_user_tracking(&e, &recipient, amount); // Receiver: increase balance
            }
            None => {
                // Burn: only update sender's tracking (tokens are destroyed)
                initialize_or_update_user_tracking(&e, &from, -(amount));
            }
        }

        (manager_fees, protocol_fees)
    }

    /// Called by token contract during external mints to enforce fee collection
    /// This prevents users from bypassing fees by acquiring tokens on external DEXes
    pub fn collect_fees_before_mint(e: Env, user: Address, amount: i128) -> (u128, u128) {
        // No auth required - this is called by the trusted token contract

        // Collect any accrued fees on user's existing balance
        let (manager_fees, protocol_fees) = collect_fees_before_action(&e, &user, amount);

        // Update user tracking with the new amount
        initialize_or_update_user_tracking(&e, &user, amount);

        (manager_fees, protocol_fees)
    }

    /// Get users with accrued fees above threshold (admin function)
    pub fn get_users_with_fees(
        e: Env,
        admin: Address,
        user_addresses: Vec<Address>,
    ) -> Vec<Address> {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        get_users_with_accrued_fees(&e, user_addresses)
    }

    /// Force collect fees from a user regardless of threshold (emergency admin function)
    pub fn force_collect_fees(e: Env, admin: Address, user: Address) -> (u128, u128) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        force_collect_fees(&e, &user)
    }

    /// Get last batch collection timestamp
    pub fn get_last_batch_collection(e: Env) -> u64 {
        get_last_batch_collection(&e)
    }
}
