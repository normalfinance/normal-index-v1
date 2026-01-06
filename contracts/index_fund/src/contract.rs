use crate::errors::IndexFundError;
use crate::events::Events;
use crate::events::IndexEvents;
use crate::storage::get_admin;
use crate::storage::get_index_vault_amount;
use crate::storage::get_token_quote;
use crate::storage::set_token_quote;
use utils::validate;

use crate::interface::{AdminInterface, IndexFundTrait, QueryInterface};
use crate::storage::get_all_rebalance_authorities;
use crate::storage::get_blacklist_status;
use crate::storage::get_rebalance_authority_status;
use crate::storage::get_whitelist_status;
use crate::storage::remove_component;
use crate::storage::set_component;
use crate::storage::set_factory;
use crate::storage::set_initial_price;
use crate::storage::set_last_rebalance_ts;
use crate::storage::set_last_updated_ts;
use crate::storage::set_public;
use crate::storage::set_rebalance_admin_status;
use crate::storage::set_total_mints;
use crate::storage::set_total_redemptions;
use crate::storage::{
    get_all_component_balances, get_all_components, get_component, get_component_balance_safe,
    get_component_registry, get_component_safe, get_factory_safe, get_initial_price,
    get_last_rebalance_ts, get_last_updated_ts, get_public, get_rebalance_threshold,
    get_total_mints, get_total_redemptions, set_component_balance,
};
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::Role;
use access_control::role::SymbolRepresentation;
use access_control::transfer::TransferOwnershipTrait;
use access_control::utils::{
    require_pause_admin_or_owner, require_pause_or_emergency_pause_admin_or_owner,
};
use soroban_sdk::Bytes;
use soroban_sdk::{
    contract, contractimpl, log, panic_with_error, token::TokenClient as SorobanTokenClient, vec,
    Address, BytesN, Env, IntoVal, Map, Symbol, Vec,
};
use token_share::{burn_shares, get_token_share, get_total_shares, mint_shares, put_token_share};
use types::index_fund::Component;
use types::index_fund::IndexParams;
use types::index_fund::{
    ComponentAction, ComponentAllocation, IndexFundInfo, IndexFundMetrics, IndexFundStatus,
    RebalanceParams, RebalanceStatus, RefactorParams,
};
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};
use utils::token::transfer_token;
use utils::token::validate_token_contracts;

#[contract]
pub struct IndexFund;

impl IndexFund {
    // __constructor
    // Initializes the ProviderSwapFeeCollector contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - factory: The address of the factory contract.
    //   - params: The address authorized to claim funds.
    pub fn __constructor(e: Env, factory: Address, serialized_asset: Bytes, params: IndexParams) {
        set_factory(&e, &factory);
        set_token_quote(&e, &params.token_quote);

        // init_admin via AccessControl
        let access_control = AccessControl::new(&e);
        if !access_control.get_role_safe(&Role::Admin).is_some() {
            access_control.set_role_address(&Role::Admin, &params.admin);
        }

        // Create the Deployer with Asset
        let deployer = e.deployer().with_stellar_asset(serialized_asset);
        let _ = deployer.deployed_address();
        // Deploy the Stellar Asset Contract
        let sac_address = deployer.deploy();

        put_token_share(&e, sac_address);
        set_public(&e, &params.is_public);
        set_initial_price(&e, &params.initial_price);

        // Execute component updates without swap operations
        IndexFund::execute_refactoring(
            &e,
            params.admin.clone(),
            RefactorParams {
                component_updates: params.components,
            },
        );
    }
}

// The `IndexTrait` trait provides the interface for interacting with a liquidity pool.
#[contractimpl]
impl IndexFundTrait for IndexFund {
    fn mint(e: Env, user: Address, amount: u128) {
        user.require_auth();

        if get_blacklist_status(&e, &user) {
            panic_with_error!(e, IndexFundError::Blacklisted);
        }

        let is_public = get_public(&e);
        if !is_public {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&user, &Role::Admin);
            let is_whitelisted = get_whitelist_status(&e, &user);

            if !is_admin && !is_whitelisted {
                panic_with_error!(e, IndexFundError::NotWhitelisted);
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

        // Deposit the token first
        transfer_token(
            &e,
            &token_quote,
            &user,
            &e.current_contract_address(),
            &(amount as i128),
        );

        // Create swap ops
        // Execute weight-based allocation
        IndexFund::execute_weight_based_mint(&e, token_quote.clone(), amount);

        //
        let total_shares = get_total_shares(&e);
        let nav = Self::get_current_nav(e.clone()) as u128;
        // FIXME: this assumes USDC is always $1
        let n_shares = crate::index::nav_amount_to_shares(&e, amount, total_shares, nav);

        // Mint share tokens
        mint_shares(&e, &user, n_shares as i128);

        // Update metrics
        set_total_mints(&e, &n_shares);
        // VolumeTracker::record_mint_volume(&e, &user, &token_quote, amount);

        // Emit enhanced mint event
        let current_time = e.ledger().timestamp();
        let nav_after = Self::get_current_nav(e.clone()) as u128;
        let total_shares_after = get_total_shares(&e);
        let share_price = Self::get_share_price(e.clone()) as u128;

        Events::new(&e).mint_executed(
            current_time,
            user.clone(),
            token_quote,
            amount,
            n_shares,
            share_price,
            nav, // Approximation of nav_before
            nav_after,
            total_shares,
            total_shares_after,
        );

        // Also emit legacy event for backward compatibility
        Events::new(&e).mint(current_time, user);
    }

    fn redeem(e: Env, user: Address, share_amount: u128) {
        user.require_auth();

        if get_blacklist_status(&e, &user) {
            panic_with_error!(e, IndexFundError::Blacklisted);
        }

        let is_public = get_public(&e);
        if !is_public {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&user, &Role::Admin);
            let is_whitelisted = get_whitelist_status(&e, &user);

            if !is_admin && !is_whitelisted {
                panic_with_error!(e, IndexFundError::NotWhitelisted);
            }
        }

        if share_amount == 0 {
            panic_with_error!(e, IndexFundError::InvalidAmount);
        }

        let current_time = e.ledger().timestamp();
        let total_shares_before = get_total_shares(&e);
        let nav_before = Self::get_current_nav(e.clone()) as u128;
        let share_price = Self::get_share_price(e.clone()) as u128;

        let user_balance = token_share::get_user_balance_shares(&e, &user);
        if user_balance < share_amount {
            panic_with_error!(e, IndexFundError::InsufficientBalance);
        }

        if total_shares_before < share_amount {
            panic_with_error!(e, IndexFundError::InsufficientBalance);
        }

        let nav_to_redeem =
            crate::index::shares_to_nav(&e, share_amount, total_shares_before, nav_before);

        let redemption_ratio = if total_shares_before > 0 {
            (share_amount * 10000) / total_shares_before
        } else {
            panic_with_error!(e, IndexFundError::InvalidSharesDetected);
        };

        let component_registry = get_component_registry(&e);
        let mut component_payouts = Map::new(&e);
        let registry_len = component_registry.len();

        for i in 0..registry_len {
            let component_token = component_registry.get_unchecked(i);
            let current_balance =
                get_component_balance_safe(&e, component_token.clone()).unwrap_or(0);

            if current_balance > 0 {
                let user_component_amount = (current_balance * redemption_ratio) / 10000;

                if user_component_amount > 0 {
                    transfer_token(
                        &e,
                        &component_token,
                        &e.current_contract_address(),
                        &user,
                        &(user_component_amount as i128),
                    );

                    let new_balance = current_balance - user_component_amount;
                    set_component_balance(&e, component_token.clone(), new_balance);

                    component_payouts.set(component_token, user_component_amount);
                }
            }
        }

        burn_shares(&e, &user, share_amount);

        let current_total_redemptions = get_total_redemptions(&e);
        set_total_redemptions(&e, &(current_total_redemptions + share_amount));

        let nav_after = Self::get_current_nav(e.clone()) as u128;
        let total_shares_after = get_total_shares(&e);

        // let redemption_usd_value =
        //     VolumeTracker::calculate_redeem_usd_value(&e, share_amount, share_price);
        // VolumeTracker::record_redeem_volume(&e, &user, redemption_usd_value);

        Events::new(&e).redemption_executed(
            current_time,
            user.clone(),
            share_amount,
            share_price,
            nav_before,
            nav_after,
            total_shares_before,
            total_shares_after,
            component_payouts,
        );

        // Also emit legacy event for backward compatibility
        Events::new(&e).redeem(current_time, user);
    }

    // fn get_factory(e: Env) -> Address {
    //     crate::storage::get_factory(&e)
    // }

    fn get_whitelist_status(e: Env, address: Address) -> bool {
        crate::storage::get_whitelist_status(&e, &address)
    }

    fn get_blacklist_status(e: Env, address: Address) -> bool {
        crate::storage::get_blacklist_status(&e, &address)
    }

    fn get_component(e: Env, token: Address) -> Component {
        crate::storage::get_component(&e, token)
    }

    fn get_component_balance(e: Env, token: Address) -> u128 {
        crate::storage::get_component_balance_safe(&e, token).unwrap_or(0)
    }

    /// Transfer shares between users
    fn transfer_shares(e: Env, from: Address, to: Address, amount: u128) {
        from.require_auth();

        // Execute the token transfer
        let share_token = get_token_share(&e);
        SorobanTokenClient::new(&e, &share_token).transfer(&from, &to, &(amount as i128));
    }

    /// Transfer shares from allowance
    fn transfer_shares_from(e: Env, spender: Address, from: Address, to: Address, amount: u128) {
        spender.require_auth();

        // Execute the token transfer from allowance
        let share_token = get_token_share(&e);
        SorobanTokenClient::new(&e, &share_token).transfer_from(
            &spender,
            &from,
            &to,
            &(amount as i128),
        );
    }
}

// The `AdminInterface` trait provides the interface for administrative actions.
#[contractimpl]
impl AdminInterface for IndexFund {
    fn refactor(e: Env, caller: Address, params: RefactorParams) {
        caller.require_auth();

        if get_blacklist_status(&e, &caller) {
            panic_with_error!(e, IndexFundError::Blacklisted);
        }

        // Validate permissions - managers (admin) can refactor anytime
        let access_control = AccessControl::new(&e);
        if !access_control.address_has_role(&caller, &Role::Admin) {
            panic_with_error!(e, IndexFundError::UnauthorizedRefactor);
        }

        // Capture pre-refactor state
        // let components_before = get_all_components(&e);
        let current_time = e.ledger().timestamp();

        // Execute component updates without swap operations
        IndexFund::execute_refactoring(&e, caller.clone(), params.clone());

        // Capture post-refactor state
        // let components_after = get_all_components(&e);

        // Update last updated timestamp (but not rebalance timestamp)
        set_last_updated_ts(&e, &current_time);

        // Emit refactor event
        // TODO: Re-enable component state capture when iteration is fixed
        // Events::new(&e).refactor_executed(
        //     current_time,
        //     caller.clone(),
        //     components_before,
        //     components_after,
        //     params.component_updates.len() as u32,
        // );
    }

    fn rebalance(e: Env, caller: Address, params: RebalanceParams) {
        caller.require_auth();

        if get_blacklist_status(&e, &caller) {
            panic_with_error!(e, IndexFundError::Blacklisted);
        }

        log!(&e, "Rebalance called by caller: {:?}", caller);

        let is_public = get_public(&e);
        if !is_public {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&caller, &Role::Admin);
            let is_whitelisted = get_whitelist_status(&e, &caller);
            let is_rebalance_authority = get_rebalance_authority_status(&e, &caller);

            log!(&e, "Is admin: {:?}", is_admin);
            log!(&e, "Is whitelisted: {:?}", is_whitelisted);
            log!(&e, "Is rebalance authority: {:?}", is_rebalance_authority);

            if !is_admin && !is_whitelisted && !is_rebalance_authority {
                panic_with_error!(e, IndexFundError::NotWhitelisted);
            }
        }

        // Check rebalance threshold timing
        let current_time = e.ledger().timestamp();
        let last_rebalance = get_last_rebalance_ts(&e);
        let threshold = get_rebalance_threshold(&e);

        if current_time < last_rebalance + threshold {
            panic_with_error!(e, IndexFundError::RebalanceTooSoon);
        }

        // Permission checks based on index type
        if is_public {
            // Public index: requires DAO proposal approval (for now, only admin)
            IndexFund::validate_public_rebalance(&e, &caller, &params);
        } else {
            // Private index: admin or rebalance authority
            IndexFund::validate_private_rebalance(&e, &caller);
        }

        log!(&e, "Rebalance validated");

        // Capture pre-rebalancing state
        let nav_before = Self::get_current_nav(e.clone()) as u128;
        let components_before = get_all_components(&e);

        log!(&e, "Nav before: {:?}", nav_before);
        log!(&e, "Components before: {:?}", components_before);

        // Execute rebalancing logic (swaps only)
        IndexFund::execute_rebalancing(&e, caller.clone(), params.clone());

        log!(&e, "Rebalancing executed");

        // Capture post-rebalancing state
        let nav_after = Self::get_current_nav(e.clone()) as u128;
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
            0, // No swaps counted here - counted in execute_rebalancing
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
        set_rebalance_admin_status(&e, &authority, status);

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

    fn set_factory(e: Env, admin: Address, factory: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_factory(&e, &factory);
    }

    fn set_initial_price(e: Env, admin: Address, initial_price: u128) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let total_shares = get_total_shares(&e);
        validate!(e, total_shares == 0, IndexFundError::InvalidSharesDetected);

        let old_price = get_initial_price(&e);
        crate::storage::set_initial_price(&e, &initial_price);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).initial_price_updated(current_time, admin, old_price, initial_price);
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

    fn convert_token_to_usd(e: Env, token: Address, amount: u128) -> u128 {
        crate::oracle::OracleUtils::convert_token_to_usd(&e, &token, amount)
    }

    fn convert_token_to_usd_safe(e: Env, token: Address, amount: u128) -> Option<u128> {
        crate::oracle::OracleUtils::convert_token_to_usd_safe(&e, &token, amount)
    }
}

// The `UpgradeableContract` trait provides the interface for upgrading the contract.
#[contractimpl]
impl UpgradeableContract for IndexFund {
    // Returns the version of the contract.
    //
    // # Returns
    //
    // The version of the contract as a u32.
    fn version() -> u32 {
        100
    }

    // Get contract type symbolic name
    fn contract_name(e: Env) -> Symbol {
        Symbol::new(&e, "IndexFund")
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
    // The emergency mode can only be set by the admin.
    //
    // # Arguments
    //
    // * `admin` - The address of the emergency admin.
    // * `value` - The value to set the emergency mode to.
    fn set_emergency_mode(e: Env, admin: Address, value: bool) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        set_emergency_mode(&e, &value);
        AccessControlEvents::new(&e).set_emergency_mode(value);
    }

    // Returns the emergency mode flag value.
    fn get_emergency_mode(e: Env) -> bool {
        get_emergency_mode(&e)
    }
}

// The `TransferableContract` trait provides the interface for transferring ownership of the contract.
#[contractimpl]
impl TransferableContract for IndexFund {
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
impl QueryInterface for IndexFund {
    // Comprehensive index information
    fn get_index_info(e: Env) -> IndexFundInfo {
        IndexFundInfo {
            address: e.current_contract_address(),
            admin_address: get_admin(&e),
            token_address: get_token_share(&e),
            total_shares: get_total_shares(&e),
            initial_price: get_initial_price(&e),
            is_public: get_public(&e),
            rebalance_threshold: get_rebalance_threshold(&e),
            last_rebalance_ts: get_last_rebalance_ts(&e),
            last_updated_ts: get_last_updated_ts(&e),
            total_mints: get_total_mints(&e),
            total_redemptions: get_total_redemptions(&e),
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
        let len = component_addresses.len();
        for i in 0..len {
            let component_address = component_addresses.get_unchecked(i);
            // Get the component balance (how much of this token the index holds)
            let balance = match get_component_balance_safe(&e, component_address.clone()) {
                Some(bal) => bal,
                None => 0u128, // If no balance stored, treat as 0
            };

            if balance > 0 {
                // Get the token price - for now we'll use a placeholder approach
                let token_price =
                    IndexFund::get_token_price_in_base_currency(&e, component_address.clone());

                // Calculate value: balance * price
                let component_value = balance.saturating_mul(token_price);
                total_value = total_value.saturating_add(component_value);
            }
        }

        total_value
    }
    // Financial metrics
    fn get_index_metrics(e: Env) -> IndexFundMetrics {
        let current_nav = IndexFund::get_current_nav(e.clone());
        let share_price = IndexFund::get_share_price(e.clone());

        IndexFundMetrics {
            total_shares: get_total_shares(&e),
            total_mints: get_total_mints(&e),
            total_redemptions: get_total_redemptions(&e),
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

        let total_value = IndexFund::get_total_index_value(e.clone());
        if total_value == 0 {
            let ip = get_initial_price(&e);
            return if ip < 0 { 0 } else { ip as u128 };
        }

        // Share price = Total Portfolio Value / Total Shares
        total_value / total_shares
    }

    fn get_current_nav(e: Env) -> u128 {
        // NAV (Net Asset Value) is the total value of all holdings
        IndexFund::get_total_index_value(e)
    }

    // Operational status
    fn get_index_status(e: Env) -> IndexFundStatus {
        let current_time = e.ledger().timestamp();
        let last_rebalance = get_last_rebalance_ts(&e);
        let threshold = get_rebalance_threshold(&e);
        let can_rebalance = current_time >= last_rebalance + threshold;

        IndexFundStatus {
            is_public: get_public(&e),
            can_rebalance,
            last_rebalance_ts: get_last_rebalance_ts(&e),
            rebalance_threshold: get_rebalance_threshold(&e),
        }
    }

    fn can_rebalance(e: Env) -> bool {
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
        let can_rebalance = current_time >= last_rebalance + threshold;
        let time_until_next = if can_rebalance {
            0
        } else {
            last_rebalance + threshold - current_time
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
        let current_nav = IndexFund::get_current_nav(e.clone());

        // Get component addresses for iteration
        let component_addresses = get_component_registry(&e);
        let len = component_addresses.len();

        for i in 0..len {
            let token = component_addresses.get_unchecked(i);
            let component = components.get(token.clone()).unwrap();

            let current_balance = get_component_balance_safe(&e, token.clone()).unwrap_or(0);
            // Target balance is based on base_nav (intended portfolio value)
            let target_balance = if current_nav > 0 {
                (current_nav * (component.weight as u128)) / 10000
            } else {
                0
            };
            // Percentage is based on current_nav (actual portfolio value)
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
impl IndexFund {
    // Helper function to calculate total value of all component holdings
    pub fn get_total_component_value(e: &Env) -> u128 {
        let mut total_value: u128 = 0;

        // Get all component addresses from registry
        let component_addresses = get_component_registry(&e);

        // Iterate through each component to calculate total portfolio value
        let len = component_addresses.len();
        for i in 0..len {
            let component_address = component_addresses.get_unchecked(i);
            // Get the component balance (how much of this token the index holds)
            let balance = match get_component_balance_safe(&e, component_address.clone()) {
                Some(bal) => bal,
                None => 0u128, // If no balance stored, treat as 0
            };

            if balance > 0 {
                // Get the token price - for now we'll use a placeholder approach
                let token_price =
                    IndexFund::get_token_price_in_base_currency(&e, component_address.clone());

                // Calculate value: balance * price
                let component_value = balance.saturating_mul(token_price);
                total_value = total_value.saturating_add(component_value);
            }
        }

        total_value
    }

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
                match IndexFund::get_price_via_factory_aggregator(&e, &factory_address, &token) {
                    Some(price) => price,
                    None => {
                        // Fall back to component weight-based pricing
                        IndexFund::get_price_from_component_weight(&e, &token)
                    }
                }
            }
            None => {
                // No factory connection, use component weight-based pricing
                IndexFund::get_price_from_component_weight(&e, &token)
            }
        }
    }

    // Helper function to get price via factory's oracle
    fn get_price_via_factory_aggregator(
        e: &Env,
        factory_address: &Address,
        token: &Address,
    ) -> Option<u128> {
        // Call factory's convert_token_to_usd_safe function
        // We query the price for 1 token unit (with 7 decimals = 10_000_000)
        let one_token_unit = 10_000_000u128;

        let result = e.try_invoke_contract::<Option<u128>, IndexFundError>(
            factory_address,
            &Symbol::new(e, "convert_token_to_usd_safe"),
            Vec::from_array(e, [token.clone().into_val(e), one_token_unit.into_val(e)]),
        );

        match result {
            Ok(Ok(Some(price_usd))) => {
                // Price returned is for 1 token unit in USD (7 decimals)
                // Convert to our internal price format (6 decimals)
                Some(price_usd / 10) // 7 decimals -> 6 decimals
            }
            _ => None, // Fall back to weight-based pricing
        }
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
            panic_with_error!(e, IndexFundError::UnauthorizedRebalance);
        }
    }

    fn validate_public_rebalance(e: &Env, caller: &Address, _params: &RebalanceParams) {
        // For now, only admin can rebalance public indexes
        // Later, add DAO proposal validation logic
        let access_control = AccessControl::new(e);
        if !access_control.address_has_role(caller, &Role::Admin) {
            panic_with_error!(e, IndexFundError::PublicRebalanceRequiresProposal);
        }

        // TODO: Validate DAO proposal approval
        // if let Some(proposal_id) = params.proposal_id {
        //     validate_dao_proposal_approval(e, proposal_id);
        // }
    }

    fn execute_rebalancing(e: &Env, admin: Address, params: RebalanceParams) {
        let start_time = e.ledger().timestamp();
        let nav_before = Self::get_current_nav(e.clone()) as u128;

        let can_rebalance = IndexFund::can_rebalance(e.clone());
        if !can_rebalance {
            panic_with_error!(e, IndexFundError::RebalanceNotAllowed);
        }

        // Generate and execute swap transactions to align current balances with target weights
        let swaps = crate::index::generate_rebalance_swaps(e, &params);
        let total_swaps = swaps.len() as u32;

        log!(&e, "Total swaps: {:?}", total_swaps);
        if total_swaps > 0 {
            log!(&e, "Executing swaps");
            let _swap_results = crate::index::execute_swaps(e, swaps);
        }

        // Capture end state for enhanced event
        let end_time = e.ledger().timestamp();
        let nav_after = Self::get_current_nav(e.clone()) as u128;
        let duration_ms = (end_time - start_time) * 1000; // Convert to milliseconds
        let performance_delta = (nav_after as i128) - (nav_before as i128);

        // Emit enhanced completion event (no components updated, only swaps)
        Events::new(e).rebalance_completed_detailed(
            end_time,
            admin,
            0, // components_updated: 0 since rebalancing doesn't update components anymore
            total_swaps,
            performance_delta,
            nav_before,
            nav_after,
            duration_ms,
        );

        // Also emit legacy event for backward compatibility
        Events::new(e).rebalance_completed(
            e.current_contract_address(),
            0, // components_updated: 0
            total_swaps,
        );
    }

    fn execute_refactoring(e: &Env, admin: Address, params: RefactorParams) {
        let mut _components_updated = 0u32;

        // Validate and execute component updates (without swaps)
        let len = params.component_updates.len();
        for i in 0..len {
            let update = params.component_updates.get_unchecked(i);
            match update.action {
                ComponentAction::Add => {
                    // Check if component already exists
                    if get_component_safe(e, update.token.clone()).is_some() {
                        panic_with_error!(e, IndexFundError::InvalidComponentAction);
                    }

                    // Require oracle for new components
                    let oracle = update.oracle.clone()
                        .unwrap_or_else(|| panic_with_error!(e, IndexFundError::MissingOracleAddress));

                    // Create component with symbol (simplified for now)
                    let component = Component {
                        asset: Symbol::new(e, "TOKEN"), // Simplified - would need proper token symbol
                        weight: update.new_weight,
                        normal: false,
                        oracle,
                    };
                    set_component(e, update.token.clone(), component);
                    crate::storage::add_component_to_registry(e, update.token.clone());
                    _components_updated += 1;

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
                    let old_weight = component.weight;
                    let final_balance =
                        get_component_balance_safe(e, update.token.clone()).unwrap_or(0);
                    let current_time = e.ledger().timestamp();

                    remove_component(e, update.token.clone());
                    _components_updated += 1;

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
                    // Check if component exists first
                    let component_exists = get_component_safe(e, update.token.clone()).is_some();

                    if !component_exists {
                        panic_with_error!(e, IndexFundError::ComponentNotFound);
                    }

                    // Get component info before updating
                    let mut component = get_component(e, update.token.clone());
                    let old_weight = component.weight;
                    component.weight = update.new_weight;

                    // Optionally update oracle if provided
                    if let Some(new_oracle) = update.oracle.clone() {
                        component.oracle = new_oracle;
                    }

                    set_component(e, update.token.clone(), component);
                    _components_updated += 1;

                    // Emit legacy event for backward compatibility
                    Events::new(e).component_weight_updated(
                        update.token.clone(),
                        old_weight,
                        update.new_weight,
                    );
                }
            }
        }

        // Validate that final weights sum to 10000
        // Calculate by iterating registry and getting components directly (avoiding get_all_components Map issues)
        let component_registry = crate::storage::get_component_registry(e);
        let registry_len = component_registry.len();

        // If no components, weights should sum to 0 (valid empty state)
        if registry_len == 0 {
            return;
        }

        let mut total_weight = 0u128;
        for i in 0..registry_len {
            let token_address = component_registry.get_unchecked(i);

            // Get component directly from storage instead of using Map
            if let Some(component) = get_component_safe(e, token_address.clone()) {
                total_weight += component.weight;
            }
        }

        // Validate that final weights sum to 10000
        if total_weight != 10000 {
            panic_with_error!(e, IndexFundError::InvalidWeightSum);
        }
    }

    fn execute_weight_based_mint(e: &Env, deposited_token: Address, deposited_amount: u128) {
        // Get all current components and their weights
        let components = crate::storage::get_all_components(e);

        if components.len() == 0 {
            // No components defined, just hold the deposited token as-is
            panic_with_error!(&e, IndexFundError::ComponentNotFound);
            // return;
        }

        let mut swaps = Vec::new(e);

        // Get component addresses for iteration
        let component_addresses = crate::storage::get_component_registry(e);

        // For each component, calculate how much of the deposited amount should be allocated
        let len = component_addresses.len();
        for i in 0..len {
            let component_token = component_addresses.get_unchecked(i);
            let component = components.get(component_token.clone()).unwrap();

            // Calculate target amount based on weight (weight is in basis points, 10000 = 100%)
            let target_amount = (deposited_amount * component.weight) / 10000;

            if target_amount > 0 {
                if component_token == deposited_token {
                    // No swap needed - the deposited token matches this component
                    // Just update the component balance directly
                    let current_balance =
                        crate::storage::get_component_balance_safe(e, component_token.clone())
                            .unwrap_or(0);
                    crate::storage::set_component_balance(
                        e,
                        component_token.clone(),
                        current_balance + target_amount,
                    );
                } else {
                    // let provider = if component.normal {
                    //     DexProvider::Normal
                    // } else {
                    //     DexProvider::Soroswap
                    // };

                    // Need to swap deposited token for component token
                    let swap = crate::index::SwapParams {
                        provider: None,
                        token_in: deposited_token.clone(),
                        token_out: component_token.clone(),
                        amount_in: target_amount,
                        amount_out_min: (target_amount * 95) / 100, // 5% slippage tolerance
                        to: e.current_contract_address(),
                    };
                    swaps.push_back(swap);
                }
            }
        }

        // Execute all swaps if any are needed
        if swaps.len() > 0 {
            let swap_results = crate::index::execute_swaps(e, swaps);

            // Update component balances based on swap results
            let mut swap_index = 0;
            let len2 = component_addresses.len();
            for i in 0..len2 {
                let component_token = component_addresses.get_unchecked(i);
                let component = components.get(component_token.clone()).unwrap();
                let target_amount = (deposited_amount * component.weight) / 10000;

                if target_amount > 0 && component_token != deposited_token {
                    // This component required a swap
                    if swap_index < swap_results.len() {
                        let amount_received = swap_results.get(swap_index).unwrap_or(0u128);
                        let current_balance =
                            crate::storage::get_component_balance_safe(e, component_token.clone())
                                .unwrap_or(0);
                        crate::storage::set_component_balance(
                            e,
                            component_token.clone(),
                            current_balance + amount_received,
                        );
                        swap_index += 1;
                    }
                }
            }
        }
    }
}
