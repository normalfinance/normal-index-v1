//! Implementation of the Soroban token interface.
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::interface::UpgradeableContract;
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use access_control::access::{AccessControl, AccessControlTrait};
use normal_rust_types::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::{Role, SymbolRepresentation};
use access_control::transfer::TransferOwnershipTrait;
use normal_rust_types::TokenError;
use soroban_sdk::token::{self, Interface as _};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, BytesN, Env, IntoVal, String, Symbol, Vec,
};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;
use utils::bump::bump_instance;

/// Get the index contract that manages this token (using existing admin relationship)
fn get_index_contract(e: &Env) -> Address {
    // The token's admin IS the index contract - elegant and simple!
    AccessControl::new(e).get_role(&Role::Admin)
}

/// Collect fees before token transfer by calling index contract
fn collect_fees_before_transfer(e: &Env, from: &Address, to: &Address, amount: i128) {
    let index_contract = get_index_contract(e);

    let _result: (u128, u128) = e.invoke_contract(
        &index_contract,
        &Symbol::new(e, "collect_fees_before_operation"),
        Vec::from_array(
            e,
            [
                from.clone().into_val(e),
                amount.into_val(e),
                Some(to.clone()).into_val(e), // Some(address) for transfers
            ],
        ),
    );
}

/// Collect fees before token mint by calling index contract
fn collect_fees_before_mint(e: &Env, to: &Address, amount: i128) {
    let index_contract = get_index_contract(e);

    let _result: (u128, u128) = e.invoke_contract(
        &index_contract,
        &Symbol::new(e, "collect_fees_before_mint"),
        Vec::from_array(e, [to.clone().into_val(e), amount.into_val(e)]),
    );
}

/// Collect fees before token burn by calling index contract
fn collect_fees_before_burn(e: &Env, from: &Address, amount: i128) {
    let index_contract = get_index_contract(e);

    let _result: (u128, u128) = e.invoke_contract(
        &index_contract,
        &Symbol::new(e, "collect_fees_before_operation"),
        Vec::from_array(
            e,
            [
                from.clone().into_val(e),
                amount.into_val(e),
                Option::<Address>::None.into_val(e), // None for burns
            ],
        ),
    );
}

fn check_nonnegative_amount(e: &Env, amount: i128) {
    if amount < 0 {
        panic_with_error!(&e, TokenError::NegativeNotAllowed);
    }
}

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(e: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        admin.require_auth();

        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, TokenError::AlreadyInitialized);
        }
        access_control.set_role_address(&Role::Admin, &admin);
        if decimal > u8::MAX.into() {
            panic_with_error!(&e, TokenError::DecimalTooLarge);
        }

        write_metadata(
            &e,
            TokenMetadata {
                decimal,
                name,
                symbol,
            },
        )
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        check_nonnegative_amount(&e, amount);
        let admin = AccessControl::new(&e).get_role(&Role::Admin);
        admin.require_auth();

        bump_instance(&e);

        // Collect fees before minting (handles all fee collection at token level)
        collect_fees_before_mint(&e, &to, amount);

        // Perform the actual mint using traditional balance functions
        receive_balance(&e, to.clone(), amount);

        TokenUtils::new(&e).events().mint(admin, to, amount);
    }
}

#[contractimpl]
impl token::Interface for Token {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        bump_instance(&e);
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(&e, amount);

        bump_instance(&e);

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        bump_instance(&e);
        read_balance(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(&e, amount);

        bump_instance(&e);

        // CRITICAL: Collect fees before transfer (prevents external DEX fee avoidance)
        collect_fees_before_transfer(&e, &from, &to, amount);

        // Perform the actual transfer using traditional balance functions
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);

        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(&e, amount);

        bump_instance(&e);

        // CRITICAL: Collect fees before transfer (prevents external DEX fee avoidance)
        collect_fees_before_transfer(&e, &from, &to, amount);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);

        TokenUtils::new(&e).events().transfer(from, to, amount)
    }

    fn burn(e: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(&e, amount);

        bump_instance(&e);

        // Collect fees before burn (user is reducing their position)
        collect_fees_before_burn(&e, &from, amount);

        // Perform the actual burn using traditional balance functions
        spend_balance(&e, from.clone(), amount);

        TokenUtils::new(&e).events().burn(from, amount);
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(&e, amount);

        bump_instance(&e);

        // Collect fees before burn_from (spender is burning from user's position)
        collect_fees_before_burn(&e, &from, amount);

        // Perform the actual burn using traditional balance functions
        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);

        TokenUtils::new(&e).events().burn(from, amount)
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}

// The `UpgradeableContract` trait provides the interface for upgrading the contract.
// This contract has no delayed upgrade. Liquidity Pool contract handles the upgrade delay.
#[contractimpl]
impl UpgradeableContract for Token {
    // Returns the version of the contract.
    //
    // # Returns
    //
    // The version of the contract as a u32.
    fn version() -> u32 {
        100
    }

    fn upgrade(e: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        e.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
    }
}

// The `TransferableContract` trait provides the interface for transferring ownership of the contract.
#[contractimpl]
impl TransferableContract for Token {
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
