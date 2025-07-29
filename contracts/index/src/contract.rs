use crate::errors::IndexError;
use crate::events::Events;
use crate::events::IndexEvents;

use crate::index::vault_amount_to_shares;
use crate::interface::{ AdminInterface, IndexTrait };
use crate::stake::Stake;
use crate::stake::{
    apply_rebase_to_insurance_fund,
    apply_rebase_to_stake,
    calculate_if_shares_lost,
    get_stake,
    if_shares_to_vault_amount,
    save_stake,
    vault_amount_to_if_shares,
    StakeAction,
};
use crate::storage::get_index_vault_amount;
use crate::storage::set_manager_fee_fraction;
use crate::storage::set_public;
use crate::storage::set_total_mints;
use crate::storage::set_factory;
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
        
        // Calculate manager fee (in basis points, so divide by 10000)
        let manager_fee = (amount * manager_fee_fraction as u128) / 10000;
     
        let protocol_fee = manager_fee / 2; // 50% to protocol, 50% to manager
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
            InsuranceFundError::InvalidIFForNewStakes
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

  
    fn get_manager_address(e: Env) -> Address {
        get_manager_address(&e)
    }

    fn get_protocol_fee_recipient(e: Env) -> Address {
        get_protocol_fee_recipient(&e)
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
