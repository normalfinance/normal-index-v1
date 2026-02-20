use crate::errors::IndexFundError;
use crate::events::Events;
use crate::events::IndexEvents;
use crate::interface::{AdminInterface, IndexFundTrait, QueryInterface};

use soroban_sdk::contractmeta;
use soroban_sdk::Bytes;
use soroban_sdk::IntoVal;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, BytesN, Env, Map, Symbol, Vec,
};
use token_share;
use token_share::Client as IndexTokenClient;

// Utils
use utils::token::transfer_token;
use utils::validate;

// Types
use types::{
    adapter::AdapterTradeParams,
    component::{Component, ComponentAllocation, RebalanceParams, RebalanceStatus, RefactorParams},
    index::{DeployIndexParams, IndexFundInfo, IndexFundMetrics, IndexFundStatus},
    volume::VolumeFeeTier,
};

// Access control
use index_access_control::access::{
    IndexAccessControl as AccessControl, IndexAccessControlTrait as AccessControlTrait,
};
use index_access_control::emergency::{get_emergency_mode, set_emergency_mode};
use index_access_control::errors::IndexAccessControlError;
use index_access_control::events::Events as AccessControlEvents;
use index_access_control::interface::TransferableContract;
use index_access_control::management::MapAddressesManagementTrait;
use index_access_control::management::MultipleAddressesManagementTrait;
use index_access_control::management::SingleAddressManagementTrait;
use index_access_control::role::{Role, SymbolRepresentation};
use index_access_control::transfer::TransferOwnershipTrait;
use index_access_control::utils::require_admin;
use index_access_control::utils::require_fee_admin_or_owner;
use index_access_control::utils::require_operations_admin_or_owner;

// Upgrade
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};

contractmeta!(
    key = "Description",
    val = "A DeFi primitive for diverisified portfolios using a basket of assets"
);

#[contract]
/// Main index-fund contract implementation.
pub struct IndexFund;

impl IndexFund {
    /// Initializes the index fund and deploys its share token.
    ///
    /// This sets access-control roles, stores immutable factory configuration,
    /// initializes core index config, and applies initial component definitions.
    ///
    /// # Arguments
    /// - `e` (`Env`): Soroban environment.
    /// - `factory` (`Address`): Index-fund factory contract address.
    /// - `index_token_wasm` (`BytesN<32>`): WASM hash used to deploy the index token.
    /// - `adapter_registry` (`Address`): Adapter registry contract address.
    /// - `factory_sequence` (`u32`): Sequence value used for token deployment salt.
    /// - `params` (`DeployIndexParams`): Initial index configuration and authorities.
    ///
    /// # Returns
    /// - `()` (unit): No direct value is returned.
    pub fn __constructor(
        e: Env,
        factory: Address,
        index_token_wasm: BytesN<32>,
        adapter_registry: Address,
        factory_sequence: u32,
        params: DeployIndexParams,
    ) {
        // params.authorities.admin.require_auth();

        // Setup access control
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &params.authorities.admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &params.authorities.emergency_admin);
        access_control.set_role_address(&Role::FeeAdmin, &params.authorities.fee_admin);
        access_control.set_role_address(&Role::RewardsAdmin, &params.authorities.rewards_admin);
        access_control
            .set_role_address(&Role::OperationsAdmin, &params.authorities.operations_admin);
        access_control.set_role_addresses(
            &Role::RebalanceAuthorities,
            &params.authorities.rebalance_authorities,
        );

        // Set constants from factory
        crate::storage::set_factory(&e, &factory);
        crate::storage::set_adapter_registry(&e, &adapter_registry);

        // Deploy the index token
        let index_token_contract =
            crate::token::create_contract(&e, index_token_wasm, &factory_sequence);
        IndexTokenClient::new(&e, &index_token_contract).initialize(
            &e.current_contract_address(),
            &7u32,
            &params.name.into_val(&e),
            &params.symbol.into_val(&e),
        );
        token_share::put_token_share(&e, index_token_contract);

        // Set config
        crate::storage::set_token_quote(&e, &params.quote_token);
        crate::storage::set_public(&e, &params.is_public);
        crate::storage::set_initial_price(&e, &params.initial_price);

        // Execute component updates without swap operations
        crate::refactor::refactor(
            &e,
            params.authorities.admin,
            RefactorParams {
                component_updates: params.components,
            },
            e.ledger().timestamp(),
        );
    }
}

// The `IndexTrait` trait provides the interface for interacting with a liquidity pool.
#[contractimpl]
impl IndexFundTrait for IndexFund {
    /// Mints index shares for a user using quote-token deposit.
    fn mint(e: Env, user: Address, amount: u128) {
        user.require_auth();

        if amount <= 0 {
            panic_with_error!(e, IndexFundError::InvalidAmount);
        }

        // Block if user is blacklisted
        if crate::storage::get_blacklist_status(&e, &user) {
            panic_with_error!(e, IndexFundError::Blacklisted);
        }

        // Block if index is private and user is not whitelisted
        if !crate::storage::get_public(&e) {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&user, &Role::Admin);
            let whitelisted = crate::storage::get_whitelist_status(&e, &user);

            if !is_admin && !whitelisted {
                panic_with_error!(e, IndexFundError::NotWhitelisted);
            }
        }

        let current_time = e.ledger().timestamp();
        let token_quote = crate::storage::get_token_quote(&e);

        // Calculate fee tier
        let month_bucket = crate::volume::get_month_bucket(current_time);
        let current_volume = crate::storage::get_user_monthly_volume(&e, &user, month_bucket);
        let (protocol_fee_bps, manager_fee_bps) =
            crate::volume::get_volume_tier_fee_bps(&e, current_volume);

        // Apply the fees
        let (total_fee, protocol_fee, manager_fee) =
            crate::fee::calculate_fee_split(amount, protocol_fee_bps, manager_fee_bps);
        let net_amount = amount.saturating_sub(total_fee);

        // Shares
        let total_shares_before = token_share::get_total_shares(&e);
        let nav_before = crate::shares::get_current_nav(&e);
        let n_shares =
            crate::shares::nav_amount_to_shares(&e, net_amount, total_shares_before, nav_before);

        // Deposit the token first
        transfer_token(
            &e,
            &token_quote,
            &user,
            &e.current_contract_address(),
            &(amount as i128),
        );

        // Collect the fees
        if protocol_fee > 0 {
            let current = crate::storage::get_accrued_protocol_fee(&e, token_quote.clone());
            crate::storage::set_accrued_protocol_fee(
                &e,
                token_quote.clone(),
                current.saturating_add(protocol_fee),
            );
        }
        if manager_fee > 0 {
            let current = crate::storage::get_accrued_manager_fee(&e, token_quote.clone());
            crate::storage::set_accrued_manager_fee(
                &e,
                token_quote.clone(),
                current.saturating_add(manager_fee),
            );
        }

        // Execute weight-based allocation only on net capital.
        crate::shares::execute_weight_based_mint(&e, token_quote.clone(), net_amount);

        token_share::mint_shares(&e, &user, n_shares as i128);

        let current_total_mints = crate::storage::get_total_mints(&e);
        crate::storage::set_total_mints(&e, &(current_total_mints + n_shares));
        crate::storage::add_user_monthly_volume(&e, &user, month_bucket, amount);

        let nav_after = crate::shares::get_current_nav(&e);
        let total_shares_after = token_share::get_total_shares(&e);
        let share_price = crate::shares::get_current_share_price(&e);

        Events::new(&e).mint(
            current_time,
            user.clone(),
            token_quote,
            amount,
            n_shares,
            share_price,
            nav_before,
            nav_after,
            total_shares_before,
            total_shares_after,
            protocol_fee,
            manager_fee,
        );
    }

    /// Redeems index shares into underlying component payouts.
    fn redeem(e: Env, user: Address, share_amount: u128) {
        user.require_auth();

        if share_amount == 0 {
            panic_with_error!(e, IndexFundError::InvalidAmount);
        }

        // Block if user is blacklisted
        if crate::storage::get_blacklist_status(&e, &user) {
            panic_with_error!(e, IndexFundError::Blacklisted);
        }

        // Block if index is private and user is not whitelisted
        if !crate::storage::get_public(&e) {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&user, &Role::Admin);
            let whitelisted = crate::storage::get_whitelist_status(&e, &user);

            if !is_admin && !whitelisted {
                panic_with_error!(e, IndexFundError::NotWhitelisted);
            }
        }

        // Fee
        let current_time = e.ledger().timestamp();
        let month_bucket = crate::volume::get_month_bucket(current_time);
        let current_volume = crate::storage::get_user_monthly_volume(&e, &user, month_bucket);
        let (protocol_fee_bps, manager_fee_bps) =
            crate::volume::get_volume_tier_fee_bps(&e, current_volume);

        // Shares
        let total_shares_before = token_share::get_total_shares(&e);
        let nav_before = crate::shares::get_current_nav(&e);
        let share_price = crate::shares::get_current_share_price(&e);

        let user_balance = token_share::get_user_balance_shares(&e, &user);
        if user_balance < share_amount {
            panic_with_error!(e, IndexFundError::InsufficientBalance);
        }

        if total_shares_before < share_amount {
            panic_with_error!(e, IndexFundError::InsufficientBalance);
        }

        let nav_to_redeem =
            crate::shares::shares_to_nav(&e, share_amount, total_shares_before, nav_before);

        let redemption_ratio = if total_shares_before > 0 {
            (share_amount * 10000) / total_shares_before
        } else {
            panic_with_error!(e, IndexFundError::InvalidSharesDetected);
        };

        let component_registry = crate::storage::get_component_registry(&e);
        let mut component_payouts = Map::new(&e);
        let registry_len = component_registry.len();

        for i in 0..registry_len {
            let component_token = component_registry.get_unchecked(i);
            let current_balance =
                crate::storage::get_component_balance_safe(&e, component_token.clone())
                    .unwrap_or(0);

            if current_balance > 0 {
                let user_component_amount = (current_balance * redemption_ratio) / 10000;

                if user_component_amount > 0 {
                    let (component_fee, manager_fee, protocol_fee) =
                        crate::fee::calculate_fee_split(
                            user_component_amount,
                            protocol_fee_bps,
                            manager_fee_bps,
                        );
                    let net_component_amount = user_component_amount.saturating_sub(component_fee);

                    transfer_token(
                        &e,
                        &component_token,
                        &e.current_contract_address(),
                        &user,
                        &(net_component_amount as i128),
                    );

                    if protocol_fee > 0 {
                        let current =
                            crate::storage::get_accrued_protocol_fee(&e, component_token.clone());
                        crate::storage::set_accrued_protocol_fee(
                            &e,
                            component_token.clone(),
                            current.saturating_add(protocol_fee),
                        );
                    }
                    if manager_fee > 0 {
                        let current =
                            crate::storage::get_accrued_manager_fee(&e, component_token.clone());
                        crate::storage::set_accrued_manager_fee(
                            &e,
                            component_token.clone(),
                            current.saturating_add(manager_fee),
                        );
                    }

                    let new_balance = current_balance - net_component_amount;
                    crate::storage::set_component_balance(&e, component_token.clone(), new_balance);

                    component_payouts.set(component_token, net_component_amount);
                }
            }
        }

        token_share::burn_shares(&e, &user, share_amount);

        let current_total_redemptions = crate::storage::get_total_redemptions(&e);
        crate::storage::set_total_redemptions(&e, &(current_total_redemptions + share_amount));
        crate::storage::add_user_monthly_volume(&e, &user, month_bucket, nav_to_redeem);

        let nav_after = crate::shares::get_current_nav(&e);
        let total_shares_after = token_share::get_total_shares(&e);

        // let redemption_usd_value =
        //     VolumeTracker::calculate_redeem_usd_value(&e, share_amount, share_price);
        // VolumeTracker::record_redeem_volume(&e, &user, redemption_usd_value);

        Events::new(&e).redemption(
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
    }

    /// Returns whitelist status for an address.
    fn get_whitelist_status(e: Env, address: Address) -> bool {
        crate::storage::get_whitelist_status(&e, &address)
    }

    /// Returns blacklist status for an address.
    fn get_blacklist_status(e: Env, address: Address) -> bool {
        crate::storage::get_blacklist_status(&e, &address)
    }

    /// Returns component metadata for a token address.
    fn get_component(e: Env, token: Address) -> Component {
        crate::storage::get_component(&e, token)
    }

    /// Returns tracked component balance for a token address.
    fn get_component_balance(e: Env, token: Address) -> u128 {
        crate::storage::get_component_balance_safe(&e, token).unwrap_or(0)
    }
}

// The `AdminInterface` trait provides the interface for administrative actions.
#[contractimpl]
impl AdminInterface for IndexFund {
    /// Applies component refactor updates.
    fn refactor(e: Env, caller: Address, params: RefactorParams) {
        caller.require_auth();

        if crate::storage::get_blacklist_status(&e, &caller) {
            panic_with_error!(e, IndexFundError::Blacklisted);
        }

        // Validate permissions - managers (admin) can refactor anytime
        let access_control = AccessControl::new(&e);
        let is_admin = access_control.address_has_role(&caller, &Role::Admin);
        let is_rebalance_authority = access_control
            .get_role_address_status_safe(&Role::RebalanceAuthorities, &caller)
            .unwrap_or(false);
        if !is_admin && !is_rebalance_authority {
            panic_with_error!(e, IndexFundError::UnauthorizedRefactor);
        }

        // Capture pre-refactor state
        let components_before = crate::storage::get_all_components(&e);
        let current_time = e.ledger().timestamp();

        // Execute component updates without swap operations
        crate::refactor::refactor(&e, caller.clone(), params.clone(), current_time);

        // Capture post-refactor state
        let components_after = crate::storage::get_all_components(&e);

        // Update last updated timestamp (but not rebalance timestamp)
        crate::storage::set_last_updated_ts(&e, &current_time);

        // Emit refactor event
        Events::new(&e).refactor(
            current_time,
            caller.clone(),
            components_before,
            components_after,
            params.component_updates.len() as u32,
        );
    }

    /// Executes portfolio rebalance operations.
    fn rebalance(e: Env, caller: Address, params: RebalanceParams) {
        caller.require_auth();

        if crate::storage::get_blacklist_status(&e, &caller) {
            panic_with_error!(e, IndexFundError::Blacklisted);
        }

        let is_public = crate::storage::get_public(&e);
        if !is_public {
            let access_control = AccessControl::new(&e);
            let is_admin = access_control.address_has_role(&caller, &Role::Admin);
            let is_whitelisted = crate::storage::get_whitelist_status(&e, &caller);
            let is_rebalance_authority = access_control
                .get_role_address_status_safe(&Role::RebalanceAuthorities, &caller)
                .unwrap_or(false);

            if !is_admin && !is_whitelisted && !is_rebalance_authority {
                panic_with_error!(e, IndexFundError::NotWhitelisted);
            }
        }

        // Check rebalance threshold timing
        let current_time = e.ledger().timestamp();
        let last_rebalance = crate::storage::get_last_rebalance_ts(&e);
        let threshold = crate::storage::get_rebalance_threshold(&e);

        if current_time < last_rebalance + threshold {
            panic_with_error!(e, IndexFundError::RebalanceTooSoon);
        }

        // Permission checks based on index type
        crate::rebalance::validate_rebalance(&e, &caller);

        // Capture pre-rebalancing state
        let nav_before = crate::shares::get_current_nav(&e);
        let components_before = crate::storage::get_all_components(&e);

        // Execute rebalancing logic (swaps only)
        crate::rebalance::rebalance(&e, caller.clone(), params.clone(), nav_before);

        // Capture post-rebalancing state
        let nav_after = crate::shares::get_current_nav(&e);
        let components_after = crate::storage::get_all_components(&e);

        // Update timestamps
        crate::storage::set_last_rebalance_ts(&e, &current_time);
        crate::storage::set_last_updated_ts(&e, &current_time);

        Events::new(&e).rebalance(
            current_time,
            caller.clone(),
            nav_before,
            nav_after,
            components_before,
            components_after,
            0, // No swaps counted here - counted in execute_rebalancing
        );
    }

    // Sets the privileged addresses.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `rewards_admin` - The address of the rewards admin.
    // * `operations_admin` - The address of the operations admin.
    // * `fee_admin` - The address of the system fee admin.
    /// Updates privileged role addresses.
    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        fee_admin: Address,
    ) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        access_control.set_role_address(&Role::RewardsAdmin, &rewards_admin);
        access_control.set_role_address(&Role::OperationsAdmin, &operations_admin);
        access_control.set_role_address(&Role::FeeAdmin, &fee_admin);
        AccessControlEvents::new(&e).set_privileged_addrs(
            rewards_admin,
            operations_admin,
            fee_admin,
        );
    }

    /// Returns configured factory address.
    fn get_factory(e: Env) -> Address {
        crate::storage::get_factory(&e)
    }

    // Returns a map of privileged roles.
    //
    // # Returns
    //
    // A map of privileged roles to their respective addresses.
    /// Returns privileged role addresses keyed by role symbol.
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>> {
        let access_control = AccessControl::new(&e);
        let mut result: Map<Symbol, Vec<Address>> = Map::new(&e);
        for role in [
            Role::Admin,
            Role::EmergencyAdmin,
            Role::RewardsAdmin,
            Role::OperationsAdmin,
            Role::FeeAdmin,
        ] {
            result.set(
                role.as_symbol(&e),
                match access_control.get_role_safe(&role) {
                    Some(v) => Vec::from_array(&e, [v]),
                    None => Vec::new(&e),
                },
            );
        }

        result.set(
            Role::RebalanceAuthorities.as_symbol(&e),
            access_control.get_role_addresses(&Role::RebalanceAuthorities),
        );

        result
    }

    /// Adds or removes a rebalance authority.
    fn set_rebalance_authority(e: Env, admin: Address, authority: Address, status: bool) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        let access_control = AccessControl::new(&e);
        access_control.set_role_address_status(&Role::RebalanceAuthorities, &authority, status);
    }

    /// Sets factory address reference.
    fn set_factory(e: Env, admin: Address, factory: Address) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        crate::storage::set_factory(&e, &factory);
    }

    /// Updates whitelist membership for an address.
    fn set_whitelist_status(e: Env, admin: Address, address: Address, status: bool) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        let old_status = crate::storage::get_whitelist_status(&e, &address);
        crate::storage::set_whitelist_status(&e, &address, status);

        Events::new(&e).whitelist_status_updated(
            e.ledger().timestamp(),
            admin,
            address,
            old_status,
            status,
        );
    }

    /// Updates blacklist membership for an address.
    fn set_blacklist_status(e: Env, admin: Address, address: Address, status: bool) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        let old_status = crate::storage::get_blacklist_status(&e, &address);
        crate::storage::set_blacklist_status(&e, &address, status);

        Events::new(&e).blacklist_status_updated(
            e.ledger().timestamp(),
            admin,
            address,
            old_status,
            status,
        );
    }

    /// Sets rebalance cooldown threshold.
    fn set_rebalance_threshold(e: Env, admin: Address, threshold: u64) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        crate::storage::set_rebalance_threshold(&e, &threshold);
    }

    /// Sets full trade-fee tier schedule.
    fn set_trade_fee_tiers(e: Env, admin: Address, tiers: Vec<VolumeFeeTier>) {
        admin.require_auth();
        require_fee_admin_or_owner(&e, &admin);

        validate!(e, tiers.len() == 4, IndexFundError::InvalidAmount);
        crate::storage::set_trade_fee_tiers(&e, tiers);
    }

    /// Updates manager fee bps for all trade tiers.
    fn set_trade_fee_tiers_manager(e: Env, admin: Address, manager_fee_bps: u32) {
        admin.require_auth();
        require_fee_admin_or_owner(&e, &admin);

        validate!(e, manager_fee_bps <= 10_000, IndexFundError::InvalidAmount);

        // crate::storage::
        // TODO:
    }

    /// Sets adapter registry address reference.
    fn set_adapter_registry(e: Env, admin: Address, registry: Address) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);
        crate::storage::set_adapter_registry(&e, &registry);
    }

    /// Claims accrued protocol fees to a destination address.
    fn claim_protocol_fees(e: Env, admin: Address, token: Address, destination: Address) -> u128 {
        admin.require_auth();
        require_fee_admin_or_owner(&e, &admin);

        let accrued = crate::storage::get_accrued_protocol_fee(&e, token.clone());
        if accrued == 0 {
            return 0;
        }
        transfer_token(
            &e,
            &token,
            &e.current_contract_address(),
            &destination,
            &(accrued as i128),
        );
        crate::storage::set_accrued_protocol_fee(&e, token, 0);
        accrued
    }

    /// Claims accrued manager fees to a destination address.
    fn claim_manager_fees(e: Env, admin: Address, token: Address, destination: Address) -> u128 {
        admin.require_auth();
        require_admin(&e, &admin);

        let accrued = crate::storage::get_accrued_manager_fee(&e, token.clone());
        if accrued == 0 {
            return 0;
        }
        transfer_token(
            &e,
            &token,
            &e.current_contract_address(),
            &destination,
            &(accrued as i128),
        );
        crate::storage::set_accrued_manager_fee(&e, token, 0);
        accrued
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
    /// Returns contract version number.
    fn version() -> u32 {
        100
    }

    // Get contract type symbolic name
    /// Returns contract type name.
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
    /// Commits a staged contract upgrade hash.
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
    /// Applies the staged contract upgrade.
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
    /// Reverts the staged contract upgrade.
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
    /// Sets emergency mode status.
    fn set_emergency_mode(e: Env, admin: Address, value: bool) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        set_emergency_mode(&e, &value);
        AccessControlEvents::new(&e).set_emergency_mode(value);
    }

    // Returns the emergency mode flag value.
    /// Returns emergency mode status.
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
    /// Commits ownership transfer for a role.
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
    /// Applies previously committed ownership transfer for a role.
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
    /// Reverts previously committed ownership transfer for a role.
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
    /// Returns pending future role address or current one when no transfer exists.
    fn get_future_address(e: Env, role_name: Symbol) -> Address {
        let access_control = AccessControl::new(&e);
        let role = Role::from_symbol(&e, role_name);
        match access_control.get_transfer_ownership_deadline(&role) {
            0 => match access_control.get_role_safe(&role) {
                Some(address) => address,
                None => panic_with_error!(&e, IndexAccessControlError::RoleNotFound),
            },
            _ => access_control.get_future_address(&role),
        }
    }
}

// Implementation of QueryInterface trait for Index contract
#[contractimpl]
impl QueryInterface for IndexFund {
    // Comprehensive index information
    /// Returns aggregated index metadata and configuration.
    fn get_index_info(e: Env) -> IndexFundInfo {
        IndexFundInfo {
            address: e.current_contract_address(),
            admin_address: AccessControl::new(&e).get_role_safe(&Role::Admin).unwrap(),
            token_address: token_share::get_token_share(&e),
            total_shares: token_share::get_total_shares(&e),
            initial_price: crate::storage::get_initial_price(&e),
            is_public: crate::storage::get_public(&e),
            rebalance_threshold: crate::storage::get_rebalance_threshold(&e),
            last_rebalance_ts: crate::storage::get_last_rebalance_ts(&e),
            last_updated_ts: crate::storage::get_last_updated_ts(&e),
            total_mints: crate::storage::get_total_mints(&e),
            total_redemptions: crate::storage::get_total_redemptions(&e),
        }
    }

    // Component and balance queries
    /// Returns all configured components.
    fn get_all_components(e: Env) -> Map<Address, Component> {
        crate::storage::get_all_components(&e)
    }

    /// Returns a single component by token.
    fn get_component_info(e: Env, token: Address) -> Component {
        crate::storage::get_component(&e, token)
    }

    /// Returns balances for all configured components.
    fn get_all_component_balances(e: Env) -> Map<Address, u128> {
        crate::storage::get_all_component_balances(&e)
    }

    // Financial metrics
    /// Returns financial metrics for the index.
    fn get_index_metrics(e: Env) -> IndexFundMetrics {
        let current_nav = IndexFund::get_current_nav(e.clone());
        let share_price = IndexFund::get_share_price(e.clone());

        IndexFundMetrics {
            total_shares: token_share::get_total_shares(&e),
            total_mints: crate::storage::get_total_mints(&e),
            total_redemptions: crate::storage::get_total_redemptions(&e),
            current_nav,
            share_price,
        }
    }

    /// Returns current index share price.
    fn get_share_price(e: Env) -> u128 {
        crate::shares::get_current_share_price(&e)
    }

    /// Returns current index NAV.
    fn get_current_nav(e: Env) -> u128 {
        crate::shares::get_current_nav(&e)
    }

    // Operational status
    /// Returns operational status information for the index.
    fn get_index_status(e: Env) -> IndexFundStatus {
        let current_time = e.ledger().timestamp();
        let last_rebalance = crate::storage::get_last_rebalance_ts(&e);
        let threshold = crate::storage::get_rebalance_threshold(&e);
        let can_rebalance = current_time >= last_rebalance + threshold;

        IndexFundStatus {
            is_public: crate::storage::get_public(&e),
            can_rebalance,
            last_rebalance_ts: crate::storage::get_last_rebalance_ts(&e),
            rebalance_threshold: crate::storage::get_rebalance_threshold(&e),
        }
    }

    /// Returns whether rebalance is currently allowed.
    fn can_rebalance(e: Env) -> bool {
        crate::rebalance::can_rebalance(&e)
    }

    // Rebalancing queries
    /// Returns detailed rebalance timing and permission status.
    fn get_rebalance_status(e: Env) -> RebalanceStatus {
        let current_time = e.ledger().timestamp();
        let last_rebalance = crate::storage::get_last_rebalance_ts(&e);
        let threshold = crate::storage::get_rebalance_threshold(&e);
        let can_rebalance = current_time >= last_rebalance + threshold;
        let time_until_next = if can_rebalance {
            0
        } else {
            last_rebalance + threshold - current_time
        };

        // Get rebalance authorities
        let rebalance_authorities = if crate::storage::get_public(&e) {
            Vec::new(&e) // Public indexes don't have individual authorities
        } else {
            let access_control = AccessControl::new(&e);
            access_control.get_role_addresses(&Role::RebalanceAuthorities)
        };

        RebalanceStatus {
            can_rebalance,
            time_until_next_rebalance: time_until_next,
            last_rebalance_ts: last_rebalance,
            rebalance_threshold: threshold,
            is_public: crate::storage::get_public(&e),
            rebalance_authorities,
        }
    }

    /// Returns whether a caller can execute rebalance.
    fn can_address_rebalance(e: Env, caller: Address) -> bool {
        let current_time = e.ledger().timestamp();
        let last_rebalance = crate::storage::get_last_rebalance_ts(&e);
        let threshold = crate::storage::get_rebalance_threshold(&e);

        if current_time < last_rebalance + threshold {
            return false;
        }

        let access_control = AccessControl::new(&e);
        let is_public = crate::storage::get_public(&e);

        if is_public {
            // Public index: only admin for now (later DAO)
            access_control.address_has_role(&caller, &Role::Admin)
        } else {
            // Private index: admin or rebalance authority
            access_control.address_has_role(&caller, &Role::Admin)
                || access_control
                    .get_role_address_status_safe(&Role::RebalanceAuthorities, &caller)
                    .unwrap_or(false)
        }
    }

    /// Returns live allocation snapshot for each component.
    fn get_component_allocation(e: Env) -> Map<Address, ComponentAllocation> {
        let mut allocations = Map::new(&e);
        let components = crate::storage::get_all_components(&e);
        let current_nav = IndexFund::get_current_nav(e.clone());

        // Get component addresses for iteration
        let component_addresses = crate::storage::get_component_registry(&e);
        let len = component_addresses.len();

        for i in 0..len {
            let token = component_addresses.get_unchecked(i);
            let component = components.get(token.clone()).unwrap();

            let current_balance =
                crate::storage::get_component_balance_safe(&e, token.clone()).unwrap_or(0);
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

    /// Returns addresses authorized for rebalancing.
    fn get_rebalance_authorities(e: Env) -> Vec<Address> {
        let access_control = AccessControl::new(&e);
        access_control.get_role_addresses(&Role::RebalanceAuthorities)
    }

    /// Returns user monthly volume for the current bucket.
    fn get_user_monthly_volume(e: Env, user: Address) -> u128 {
        let month = crate::volume::get_month_bucket(e.ledger().timestamp());
        crate::storage::get_user_monthly_volume(&e, &user, month)
    }

    /// Returns configured trade fee tiers.
    fn get_trade_fee_tiers(e: Env) -> Vec<VolumeFeeTier> {
        crate::storage::get_trade_fee_tiers(&e)
    }
}
