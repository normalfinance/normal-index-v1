use crate::errors::IndexError;
use crate::events::Events;
use crate::events::IndexEvents;

use crate::index::vault_amount_to_shares;
use crate::interface::{ AdminInterface, IndexTrait, QueryInterface, IndexInfo, IndexMetrics, IndexStatus };
use crate::stake::Stake;
use crate::storage::get_index_vault_amount;
use crate::storage::set_manager_fee_fraction;
use crate::storage::set_public;
use crate::storage::set_total_mints;
use crate::storage::set_factory;
use crate::storage::{
    get_factory,
    get_base_nav,
    get_initial_price,
    set_base_nav,
    set_initial_price,
    get_public,
    get_rebalance_threshold,
    set_rebalance_threshold,
    get_last_rebalance_ts,
    get_last_updated_ts,
    get_total_mints,
    get_total_redemptions,
    get_component,
    get_component_balance,
    get_last_fee_collection,
    get_whitelist_status,
    get_blacklist_status,
    set_whitelist_status,
    set_blacklist_status,
};
use crate::storage::{
    get_manager_fee_fraction,
    get_manager_address,
    get_protocol_fee_recipient,
    set_manager_address,
    set_protocol_fee_recipient,
    get_accumulated_manager_fees,
    get_accumulated_protocol_fees,
    set_accumulated_manager_fees,
    set_accumulated_protocol_fees,
    set_last_fee_collection,
    get_total_fees,
    set_total_fees,
};
use crate::storage::{
    get_insurance_vault_amount,
    get_is_killed_mint,
    get_is_killed_redeem,
    get_is_killed_rebalance,
    get_max_shares,
    get_shares_base,
    get_token,
    get_total_shares,
    get_unstaking_period,
    put_token,
    set_is_killed_mint,
    set_is_killed_redeem,
    set_is_killed_rebalance,
    set_max_shares,
    set_total_shares,
    set_unstaking_period,
    get_base_nav,
    get_initial_price,
    get_public,
    get_last_rebalance_ts,
    get_last_updated_ts,
    get_total_mints,
    get_total_redemptions,
    get_rebalance_threshold,
    get_all_components,
    get_component,
    get_all_component_balances,
    get_component_balance,
    get_component_registry,
    get_component_balance_safe,
    get_factory,
    get_factory_safe,
    Component,
};
use access_control::access::{ AccessControl, AccessControlTrait };
use access_control::emergency::{ get_emergency_mode, set_emergency_mode };
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::{ MultipleAddressesManagementTrait, SingleAddressManagementTrait };
use access_control::role::Role;
use access_control::role::SymbolRepresentation;
use access_control::transfer::TransferOwnershipTrait;
use access_control::utils::{
    require_pause_admin_or_owner,
    require_pause_or_emergency_pause_admin_or_owner,
};
use soroban_sdk::auth::{ ContractContext, InvokerContractAuthEntry, SubContractInvocation };
use soroban_sdk::{
    contract,
    contractimpl,
    log,
    panic_with_error,
    vec,
    Address,
    BytesN,
    Env,
    IntoVal,
    Symbol,
    Vec,
    Map,
};
use token_share::mint_shares;
use token_share::put_token_share;
use token_share::Client as LPTokenClient;
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{ apply_upgrade, commit_upgrade, revert_upgrade };
use utils::math::safe_math::SafeMath;
use utils::token::transfer_token;
use utils::token::validate_token_contracts;
use utils::validate;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::String;

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
        components: Vec<Address>
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
        
        set_last_fee_collection(&e, &e.ledger().timestamp());

    }

    // Helper function to calculate and collect manager fees
    fn collect_manager_fees(e: &Env, amount: u128, user: &Address, token: &Address) -> u128 {
        let manager_fee_fraction = get_manager_fee_fraction(e);
        
        if manager_fee_fraction == 0 {
            return amount; // No fees to collect
        }
        
        // Calculate manager fee (in basis points, so divide by 20000, since two halves)
        let manager_fee = (amount * manager_fee_fraction as u128) / 20000;
     
        let protocol_fee = (amount * manager_fee_fraction as u128) / 20000;
        let total_fee = manager_fee + protocol_fee;
        
        // Accumulate fees
        let current_manager_fees = get_accumulated_manager_fees(e);
        let current_protocol_fees = get_accumulated_protocol_fees(e);
        let current_total_fees = get_total_fees(e);
        
        set_accumulated_manager_fees(e, &(current_manager_fees + manager_fee));
        set_accumulated_protocol_fees(e, &(current_protocol_fees + protocol_fee));
        set_total_fees(e, &(current_total_fees + total_fee));
        
        set_last_fee_collection(e, &e.ledger().timestamp());
        
        Events::new(e).fee_collected(user.clone(), token.clone(), amount, manager_fee, protocol_fee);
        
        amount - total_fee
    }
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
        max_slippage: Option<u64>
    ) {
        user.require_auth();

        if get_is_killed_mint(&e) {
            panic_with_error!(e, IndexError::IndexMintKilled);
        }

        validate_token_contracts(&e, &vec![&e, token.clone()]);

        // ...

        let total_shares = get_total_shares(&e);

        let vault_amount = get_index_vault_amount(&e, &token);
        let insurance_vault_amount = get_insurance_vault_amount(&e);

        validate!(
            &e,
            !(insurance_vault_amount == 0 && total_shares != 0),
            IndexError::InvalidIFForNewStakes
        );

        // Collect manager fees from the deposit amount
        let amount_after_fees = Index::collect_manager_fees(&e, amount, &user, &token);
        
        let n_shares = vault_amount_to_shares(&e, amount_after_fees, total_shares, vault_amount);

        // Configure swaps
        let swaps_chain: Vec<(Vec<Address>, BytesN<32>, Address)> = Vec::new(&e);

        // Execute swaps
        // Deposit the token
        transfer_token(&e, &token, &user, &e.current_contract_address(), &(amount as i128));
        if swaps_chain.len() == 0 {
            panic_with_error!(&e, IndexError::PathIsEmpty);
        }

        // execute swaps

        // Mint share tokens
        let value = match destination {
            Some(v) => v,
            None => user,
        };
        mint_shares(&e, &value, n_shares as i128);

        // Metrics
        set_total_mints(&e, &n_shares);
    }

    fn redeem(e: Env, user: Address, _share_amount: u128) {
        user.require_auth();

        if get_is_killed_redeem(&e) {
            panic_with_error!(e, IndexError::IndexRedeemKilled);
        }
    }

    fn get_token(e: Env) -> Address {
        crate::storage::get_token(&e)
    }

    fn get_factory(e: Env) -> Address {
        crate::storage::get_factory(&e)
    }

    fn get_base_nav(e: Env) -> i128 {
        crate::storage::get_base_nav(&e)
    }

    fn get_initial_price(e: Env) -> i128 {
        crate::storage::get_initial_price(&e)
    }

    fn get_nav(e: Env) -> i128 {
        let base_nav = crate::storage::get_base_nav(&e);
        let total_shares = crate::storage::get_total_shares(&e);
        if total_shares == 0 {
            return base_nav;
        }
        
        let token = crate::storage::get_token(&e);
        let vault_amount = crate::storage::get_index_vault_amount(&e, &token) as i128;
        vault_amount
    }

    fn get_price(e: Env) -> i128 {
        let nav = Self::get_nav(e.clone());
        let total_shares = crate::storage::get_total_shares(&e);
        if total_shares == 0 {
            return crate::storage::get_initial_price(&e);
        }
        nav / (total_shares as i128)
    }

    fn get_total_shares(e: Env) -> u128 {
        crate::storage::get_total_shares(&e)
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

        put_token(&e, &token);
    }

    // Sets the unstaking period.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `unstaking_period` - The new unstaking period.
    fn set_unstaking_period(e: Env, admin: Address, unstaking_period: u64) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        set_unstaking_period(&e, &unstaking_period);
    }

    // Sets the max shares the Insurance Fund can have.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `max_shares` - The max number of shares.
    fn set_max_shares(e: Env, admin: Address, max_shares: u128) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        set_max_shares(&e, &max_shares);
    }

    fn rebalance(e: Env, admin: Address) {
        admin.require_auth();

        if get_is_killed_rebalance(&e) {
            panic_with_error!(e, IndexError::IndexRebalanceKilled);
        }
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
        
        
        Events::new(&e).manager_address_updated(old_manager, manager);
    }

  
    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let old_recipient = get_protocol_fee_recipient(&e);
        set_protocol_fee_recipient(&e, &recipient);
        

        Events::new(&e).protocol_fee_recipient_updated(old_recipient, recipient);
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

    fn set_base_nav(e: Env, admin: Address, base_nav: i128) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_base_nav(&e, &base_nav);
    }

    fn set_initial_price(e: Env, admin: Address, initial_price: i128) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_initial_price(&e, &initial_price);
    }

    fn set_public_status(e: Env, admin: Address, public: bool) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_public(&e, &public);
    }

    fn set_whitelist_status(e: Env, admin: Address, address: Address, status: bool) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_whitelist_status(&e, &address, status);
    }

    fn set_blacklist_status(e: Env, admin: Address, address: Address, status: bool) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_blacklist_status(&e, &address, status);
    }

    fn set_manager_fee_fraction(e: Env, admin: Address, fee_fraction: u32) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_manager_fee_fraction(&e, &fee_fraction);
    }

    fn set_rebalance_threshold(e: Env, admin: Address, threshold: u64) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_rebalance_threshold(&e, &threshold);
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
            0 =>
                match access_control.get_role_safe(&role) {
                    Some(address) => address,
                    None => panic_with_error!(&e, AccessControlError::RoleNotFound),
                }
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
            token_address: get_token(&e),
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
    
    fn get_component_balance(e: Env, token: Address) -> u128 {
        get_component_balance(&e, token)
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
                let token_price = Index::get_token_price_in_base_currency(&e, component_address.clone());
                
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
            return get_initial_price(&e);
        }
        
        let total_value = Index::get_total_index_value(e.clone());
        if total_value == 0 {
            return get_initial_price(&e);
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
        let can_rebalance = (current_time >= last_rebalance + threshold) && !get_is_killed_rebalance(&e);
        
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
            },
            None => {
                // No factory connection, use component weight-based pricing
                Index::get_price_from_component_weight(&e, &token)
            }
        }
    }
    
    // Helper function to get price via factory aggregator (simulation)
    fn get_price_via_factory_aggregator(e: &Env, _factory_address: &Address, token: &Address) -> Option<u128> {
        // FUTURE IMPLEMENTATION:
        // This would make an actual call to the factory's aggregator to get current market prices
        // For example:
        // let factory_client = FactoryClient::new(e, factory_address);
        // let base_currency = Address::from_str(e, "USDC_CONTRACT_ADDRESS");
        // let price_result = factory_client.get_spot_price(token, &base_currency, &1_000_000u128);
        
        // For now, simulate different prices for demonstration
        let token_str = token.to_string();
        
        // Mock prices based on token address patterns (for demonstration)
        if token_str.contains("usdc") || token_str.contains("USDC") {
            Some(1_000_000u128) // 1 USDC = 1.000000 (6 decimals)
        } else if token_str.contains("xlm") || token_str.contains("XLM") {
            Some(120_000u128)   // 1 XLM = 0.12 USDC (simulated price)
        } else if token_str.contains("btc") || token_str.contains("BTC") {
            Some(45_000_000_000u128) // 1 BTC = 45,000 USDC (simulated price)
        } else if token_str.contains("eth") || token_str.contains("ETH") {
            Some(2_500_000_000u128)  // 1 ETH = 2,500 USDC (simulated price)
        } else {
            None // Unknown token, fall back to weight-based pricing
        }
    }
    
    // Helper function to get price based on component weight
    fn get_price_from_component_weight(e: &Env, token: &Address) -> u128 {
        // Get component information to use weight as a price indicator
        match get_component_safe(e, token.clone()) {
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
            },
            None => {
                // Token not found in components, use default price
                1_000_000u128 // 1.0 with 6 decimals
            }
        }
    }
}
