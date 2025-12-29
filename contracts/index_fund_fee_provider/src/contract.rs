use crate::errors::IndexFundFeeProviderError;
use crate::events::Events;
use crate::events::IndexFundFeeProviderEvents;
use crate::interface::{AdminInterface, IndexFundFeeProviderTrait};
use crate::storage::get_fee_token;
use crate::storage::get_is_killed_fee;
use crate::storage::get_mint_fee;
use crate::storage::get_protocol_fee;
use crate::storage::get_redeem_fee;
use crate::storage::put_fee_token;
use crate::storage::set_is_killed_fee;
use crate::storage::set_mint_fee;
use crate::storage::set_protocol_fee;
use crate::storage::set_redeem_fee;
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::{Role, SymbolRepresentation};
use access_control::transfer::TransferOwnershipTrait;
use access_control::utils::require_fee_admin_or_owner;
use access_control::utils::require_pause_admin_or_owner;
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::IntoVal;
use soroban_sdk::Map;
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, BytesN, Env, Symbol, Vec};
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};
use utils::math::safe_math::SafeMath;

#[contract]
pub struct IndexFundFeeProvider;

#[contractimpl]
impl IndexFundFeeProvider {
    // __constructor
    // Initializes the factory by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The address to be assigned the Admin role.
    //   - privileged_addrs: The address to be assigned the EmergencyAdmin role.
    pub fn __constructor(
        e: Env,
        admin: Address,
        privileged_addrs: (Address, Address, Address),
        fee_token: Address,
    ) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &privileged_addrs.0);
        access_control.set_role_address(&Role::FeeAdmin, &privileged_addrs.1);
        access_control.set_role_address(&Role::PauseAdmin, &privileged_addrs.2);

        put_fee_token(&e, fee_token);
    }
}

#[contractimpl]
impl IndexFundFeeProviderTrait for IndexFundFeeProvider {
    fn mint(
        e: Env,
        user: Address,
        index_fund: Address,
        token: Address,
        amount: u128,
        destination: Option<Address>,
    ) {
        user.require_auth();

        if !get_is_killed_fee(&e) {
            let mint_fee = get_mint_fee(&e);
            SorobanTokenClient::new(&e, &get_fee_token(&e)).transfer(
                &user,
                &e.current_contract_address(),
                &(mint_fee as i128),
            );

            // Increment protocol fee
            let protocol_fee = get_protocol_fee(&e, token.clone());
            set_protocol_fee(&e, token.clone(), protocol_fee.safe_add(&e, mint_fee));

            Events::new(&e).charge_provider_fee(token.clone(), mint_fee);
        }

        match e.try_invoke_contract::<u128, soroban_sdk::Error>(
            &index_fund,
            &Symbol::new(&e, "mint"),
            Vec::from_array(
                &e,
                [
                    user.into_val(&e),
                    token.into_val(&e),
                    amount.into_val(&e),
                    destination.into_val(&e),
                ],
            ),
        ) {
            Ok(Err(_)) | Err(_) => {
                panic_with_error!(e, IndexFundFeeProviderError::IndexFundCallFailure);
            }
            Ok(Ok(_)) => {}
        }
    }

    fn redeem(
        e: Env,
        user: Address,
        index_fund: Address,
        index_token: Address,
        share_amount: u128,
    ) {
        user.require_auth();

        if !get_is_killed_fee(&e) {
            let redeem_fee = get_redeem_fee(&e);
            SorobanTokenClient::new(&e, &get_fee_token(&e)).transfer(
                &user,
                &e.current_contract_address(),
                &(redeem_fee as i128),
            );

            // Increment protocol fee
            let protocol_fee = get_protocol_fee(&e, index_token.clone());
            set_protocol_fee(
                &e,
                index_token.clone(),
                protocol_fee.safe_add(&e, redeem_fee),
            );

            Events::new(&e).charge_provider_fee(index_token, redeem_fee);
        }

        match e.try_invoke_contract::<u128, soroban_sdk::Error>(
            &index_fund,
            &Symbol::new(&e, "redeem"),
            Vec::from_array(
                &e,
                [user.clone().into_val(&e), share_amount.clone().into_val(&e)],
            ),
        ) {
            Ok(Err(_)) | Err(_) => {
                panic_with_error!(e, IndexFundFeeProviderError::IndexFundCallFailure);
            }
            Ok(Ok(_)) => {}
        }
    }
}

#[contractimpl]
impl AdminInterface for IndexFundFeeProvider {
    fn set_fee_token(e: Env, admin: Address, token: Address) -> Address {
        admin.require_auth();
        require_fee_admin_or_owner(&e, &admin);

        put_fee_token(&e, token.clone());

        token
    }

    fn set_mint_fee(e: Env, admin: Address, fee: u128) -> u128 {
        admin.require_auth();
        require_fee_admin_or_owner(&e, &admin);

        set_mint_fee(&e, &fee);

        Events::new(&e).set_mint_fee(fee);

        fee
    }

    fn set_redeem_fee(e: Env, admin: Address, fee: u128) -> u128 {
        admin.require_auth();
        require_fee_admin_or_owner(&e, &admin);

        set_redeem_fee(&e, &fee);

        Events::new(&e).set_redeem_fee(fee);

        fee
    }

    fn get_fee_config(e: Env) -> (u128, u128) {
        (get_mint_fee(&e), get_redeem_fee(&e))
    }

    // Returns the protocol fees accumulated in the pool.
    fn get_protocol_fees_by_token(e: Env, token: Address) -> u128 {
        get_protocol_fee(&e, token)
    }

    // Claims the protocol fees accumulated in the provider.
    fn claim_protocol_fees(e: Env, admin: Address, token: Address, destination: Address) -> u128 {
        admin.require_auth();
        require_fee_admin_or_owner(&e, &admin);

        let fees = get_protocol_fee(&e, token.clone());

        if fees == 0 {
            return 0;
        }

        if fees > 0 {
            SorobanTokenClient::new(&e, &token.clone()).transfer(
                &e.current_contract_address(),
                &destination,
                &(fees as i128),
            );
            set_protocol_fee(&e, token.clone(), 0);
            Events::new(&e).claim_fee(token, fees, destination.clone());
        }

        fees
    }

    // Sets the privileged addresses.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `fee_admin` - The address of the rewards admin.
    // * `pause_admin` - The address of the pause admin.
    fn set_privileged_addrs(e: Env, admin: Address, fee_admin: Address, pause_admin: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        access_control.set_role_address(&Role::FeeAdmin, &fee_admin);
        access_control.set_role_address(&Role::PauseAdmin, &pause_admin);
    }

    // Returns a map of privileged roles.
    //
    // # Returns
    //
    // A map of privileged roles to their respective addresses.
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>> {
        let access_control = AccessControl::new(&e);
        let mut result: Map<Symbol, Vec<Address>> = Map::new(&e);
        for role in [
            Role::Admin,
            Role::EmergencyAdmin,
            Role::FeeAdmin,
            Role::PauseAdmin,
        ] {
            result.set(
                role.as_symbol(&e),
                match access_control.get_role_safe(&role) {
                    Some(v) => Vec::from_array(&e, [v]),
                    None => Vec::new(&e),
                },
            );
        }

        result
    }

    // Stops the provider fees instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_fee(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_fee(&e, &true);
        Events::new(&e).kill_fee();
    }

    // Resumes the pool fees.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_fee(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_fee(&e, &false);
        Events::new(&e).unkill_fee();
    }

    // Get fee killswitch status.
    fn get_is_killed_fee(e: Env) -> bool {
        get_is_killed_fee(&e)
    }
}
// The `UpgradeableContract` trait provides the interface for upgrading the contract.
#[contractimpl]
impl UpgradeableContract for IndexFundFeeProvider {
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
        Symbol::new(&e, "IndexFundFeeProvider")
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

// The `TransferableContract` trait provides the interface for transferring ownership of the contract.
#[contractimpl]
impl TransferableContract for IndexFundFeeProvider {
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
