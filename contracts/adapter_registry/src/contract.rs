use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, Address, BytesN, Env, Map, Symbol, Vec,
};

use crate::errors::AdapterRegistryError;
use crate::interface::AdapterRegistryTrait;
use crate::storage;

// Access control
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::{Role, SymbolRepresentation};
use access_control::transfer::TransferOwnershipTrait;
use access_control::utils::require_operations_admin_or_owner;

// Upgrade
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};

contractmeta!(
    key = "Description",
    val = "A registry tracking supported adapters for use with Index Funds"
);

#[contract]
pub struct AdapterRegistry;

#[contractimpl]
impl AdapterRegistry {
    /// Initializes the registry with its admin address.
    pub fn __constructor(
        e: Env,
        admin: Address,
        emergency_admin: Address,
        operations_admin: Address,
    ) {
        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, AdapterRegistryError::AlreadyInitialized);
        }

        access_control.set_role_address(&Role::Admin, &admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &emergency_admin);
        access_control.set_role_address(&Role::OperationsAdmin, &operations_admin);
    }
}

#[contractimpl]
impl AdapterRegistryTrait for AdapterRegistry {
    // Sets the privileged addresses.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `operations_admin` - The address of the operations admin.
    /// Updates privileged role addresses on the factory access-control module.
    fn set_privileged_addrs(e: Env, admin: Address, operations_admin: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        access_control.set_role_address(&Role::OperationsAdmin, &operations_admin);
        AccessControlEvents::new(&e).set_adapter_registry_privileged_addrs(operations_admin);
    }

    /// Adds or updates a named adapter mapping.
    fn set_adapter(e: Env, admin: Address, name: Symbol, adapter: Address) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        if let Some(assigned_name) = storage::get_name_by_adapter(&e, &adapter) {
            if assigned_name != name {
                panic_with_error!(&e, AdapterRegistryError::AdapterAddressAlreadyAssigned);
            }
        }

        if let Some(old_adapter) = storage::get_adapter_by_name(&e, &name) {
            if old_adapter != adapter {
                storage::remove_name_by_adapter(&e, &old_adapter);
            }
        }

        storage::set_adapter_by_name(&e, &name, &adapter);
        storage::set_name_by_adapter(&e, &adapter, &name);
        storage::add_adapter_name(&e, &name);
    }

    /// Returns adapter address for a required adapter name.
    fn get_adapter(e: Env, name: Symbol) -> Address {
        match storage::get_adapter_by_name(&e, &name) {
            Some(address) => address,
            None => panic_with_error!(&e, AdapterRegistryError::AdapterNameNotFound),
        }
    }

    /// Returns adapter address for a name when present.
    fn get_adapter_safe(e: Env, name: Symbol) -> Option<Address> {
        storage::get_adapter_by_name(&e, &name)
    }

    /// Returns the registered name for an adapter address.
    fn get_adapter_name(e: Env, adapter: Address) -> Symbol {
        match storage::get_name_by_adapter(&e, &adapter) {
            Some(name) => name,
            None => panic_with_error!(&e, AdapterRegistryError::AdapterAddressNotFound),
        }
    }

    /// Returns all registered adapter mappings.
    fn get_adapters(e: Env) -> Map<Symbol, Address> {
        storage::get_all_adapters(&e)
    }

    /// Returns all privileged role addresses keyed by role symbol.
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>> {
        let access_control = AccessControl::new(&e);
        let mut result: Map<Symbol, Vec<Address>> = Map::new(&e);
        for role in [Role::Admin, Role::EmergencyAdmin, Role::OperationsAdmin] {
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
}

#[contractimpl]
impl UpgradeableContract for AdapterRegistry {
    // version
    // Returns the current version number of the contract.
    //
    // Returns:
    //   - A u32 representing the version.
    /// Returns the contract version number.
    fn version() -> u32 {
        100
    }

    // Get contract type symbolic name
    /// Returns the contract type name.
    fn contract_name(e: Env) -> Symbol {
        Symbol::new(&e, "AdapterRegistry")
    }

    // commit_upgrade
    // Commits a new WASM hash as a pending upgrade.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - new_wasm_hash: The new WASM hash (BytesN<32>) to be committed.
    /// Commits a staged upgrade hash for later application.
    fn commit_upgrade(e: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        commit_upgrade(&e, &new_wasm_hash);
        UpgradeEvents::new(&e).commit_upgrade(Vec::from_array(&e, [new_wasm_hash.clone()]));
    }

    // apply_upgrade
    // Applies the previously committed upgrade.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //
    // Returns:
    //   - The new WASM hash (BytesN<32>) that was applied.
    /// Applies the currently staged contract upgrade.
    fn apply_upgrade(e: Env, admin: Address) -> BytesN<32> {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        let new_wasm_hash = apply_upgrade(&e);
        UpgradeEvents::new(&e).apply_upgrade(Vec::from_array(&e, [new_wasm_hash.clone()]));
        new_wasm_hash
    }

    // revert_upgrade
    // Reverts a pending upgrade that has not yet been applied.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    /// Reverts the currently staged contract upgrade.
    fn revert_upgrade(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        revert_upgrade(&e);
        UpgradeEvents::new(&e).revert_upgrade();
    }

    // set_emergency_mode
    // Sets or unsets emergency mode for instant upgrades.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - emergency_admin: The emergency admin address (must be authorized).
    //   - value: Boolean indicating whether to enable (true) or disable (false) emergency mode.
    /// Toggles emergency mode for upgrade behavior.
    fn set_emergency_mode(e: Env, emergency_admin: Address, value: bool) {
        emergency_admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&emergency_admin, &Role::EmergencyAdmin);
        set_emergency_mode(&e, &value);

        AccessControlEvents::new(&e).set_emergency_mode(value);
    }

    // get_emergency_mode
    // Returns the current emergency mode state.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //
    // Returns:
    //   - A boolean indicating whether emergency mode is active.
    /// Returns current emergency mode status.
    fn get_emergency_mode(e: Env) -> bool {
        get_emergency_mode(&e)
    }
}

#[contractimpl]
impl TransferableContract for AdapterRegistry {
    // commit_transfer_ownership
    // Commits to transferring ownership of a given role.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - role_name: The symbol representing the role (e.g., "Admin" or "EmergencyAdmin").
    //   - new_address: The new address to assume the role.
    /// Commits ownership transfer for a role.
    fn commit_transfer_ownership(e: Env, admin: Address, role_name: Symbol, new_address: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        access_control.commit_transfer_ownership(&role, &new_address);
        AccessControlEvents::new(&e).commit_transfer_ownership(role, new_address);
    }

    // apply_transfer_ownership
    // Applies the pending ownership transfer for a role.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - role_name: The symbol representing the role.
    /// Applies a previously committed ownership transfer.
    fn apply_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name.clone());
        // let old_address = access_control.get_role(&role);
        let new_address = access_control.apply_transfer_ownership(&role);

        // Emit factory admin updated event if this is an Admin role transfer
        // if role_name == Symbol::new(&e, "Admin") {
        //     let current_time = e.ledger().timestamp();
        //     Events::new(&e).factory_admin_updated(current_time, old_address, new_address.clone());
        // }

        AccessControlEvents::new(&e).apply_transfer_ownership(role, new_address);
    }

    // revert_transfer_ownership
    // Reverts a pending ownership transfer for a role.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - role_name: The symbol representing the role.
    /// Reverts a previously committed ownership transfer.
    fn revert_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        access_control.revert_transfer_ownership(&role);
        AccessControlEvents::new(&e).revert_transfer_ownership(role);
    }

    // get_future_address
    // Returns the pending future address for a role if an ownership transfer is committed;
    // otherwise, returns the current role address.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - role_name: The symbol representing the role.
    //
    // Returns:
    //   - The Address scheduled to assume the role, or the current address if none pending.
    /// Returns pending transfer target or current role address.
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
