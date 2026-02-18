use crate::events::Events;
use crate::events::FactoryConfigEvents;
use crate::events::FactoryEvents;
use crate::index_utils::get_index_salt;
use crate::interface::{AdminInterface, IndexFundFactoryTrait};
use crate::storage::get_index_contract_wasm;
use crate::storage::get_swap_utility;
use crate::storage::set_index_contract_wasm;
use crate::storage::set_swap_utility;
use crate::storage::{
    add_deployed_index, get_all_deployed_indexes, get_contract_sequence, get_deployed_indexes,
    set_contract_sequence,
};
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::{Role, SymbolRepresentation};
use access_control::transfer::TransferOwnershipTrait;
use access_control::utils::require_admin;
use soroban_sdk::Bytes;
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, BytesN, Env, Symbol, Vec,
};
use types::index_fund::IndexParams;
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};

#[contract]
pub struct IndexFundFactory;

// Factory configuration struct for query methods
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactoryConfig {
    pub swap_utility: Address,
    pub index_contract_wasm: BytesN<32>,
}

#[contractimpl]
impl IndexFundFactory {
    // __constructor
    // Initializes the factory by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The address to be assigned the Admin role.
    //   - emergency_admin: The address to be assigned the EmergencyAdmin role.
    //   - swap_utility: The address of the swap swap_utility contract.
    //   - index_contract_wasm: The WASM hash (BytesN<32>) for the swap fee contract.
    pub fn __constructor(
        e: Env,
        admin: Address,
        emergency_admin: Address,
        swap_utility: Address,
        index_contract_wasm: BytesN<32>,
    ) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);
        access_control.commit_transfer_ownership(&Role::EmergencyAdmin, &emergency_admin);
        access_control.apply_transfer_ownership(&Role::EmergencyAdmin);

        set_swap_utility(&e, &swap_utility);
        set_index_contract_wasm(&e, &index_contract_wasm);
    }
}

#[contractimpl]
impl IndexFundFactoryTrait for IndexFundFactory {
    // deploy_index_contract
    // Deploys a new Index Fund contract instance.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - params: ... (params.admin must be authorized).
    //
    // Returns:
    //   - The address of the newly deployed Index Fund contract.
    fn deploy_index_contract(e: Env, serialized_asset: Bytes, params: IndexParams) -> Address {
        params.admin.require_auth();

        let sequence = get_contract_sequence(&e, params.admin.clone());
        set_contract_sequence(&e, params.admin.clone(), sequence + 1);

        let salt = get_index_salt(&e, &params.admin, &sequence);

        let address = e.deployer().with_current_contract(salt).deploy_v2(
            get_index_contract_wasm(&e),
            (
                e.current_contract_address(),
                serialized_asset.clone(),
                params.clone(),
            ),
        );

        // Add to index registry
        add_deployed_index(&e, &params.admin, &address);

        // Emit enhanced deployment event
        let current_time = e.ledger().timestamp();
        let initial_components = Vec::new(&e); // Empty initially
        let initial_weights = Vec::new(&e); // Empty initially
        let initial_price = 0; // TODO: Get from contract parameters
        let is_public = false; // TODO: Get from contract parameters

        // TODO: fix correctly
        Events::new(&e).index_deployed(
            current_time,
            params.admin.clone(),
            address.clone(), // index_address
            params.admin.clone(),
            params.admin.clone(), // manager (using fee_destination as manager for now)
            initial_components,
            initial_weights,
            initial_price,
            is_public,
        );

        address
    }
}

#[contractimpl]
impl AdminInterface for IndexFundFactory {
    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    // Query Methods - Factory Configuration
    fn get_factory_config(e: Env) -> FactoryConfig {
        FactoryConfig {
            swap_utility: get_swap_utility(&e),
            index_contract_wasm: get_index_contract_wasm(&e),
        }
    }

    // Individual getters for factory configuration
    fn get_swap_utility(e: Env) -> Address {
        get_swap_utility(&e)
    }

    fn get_index_contract_wasm(e: Env) -> BytesN<32> {
        get_index_contract_wasm(&e)
    }

    // Index Registry Query Methods
    fn get_deployed_indexes(e: Env, operator: Address) -> Vec<Address> {
        get_deployed_indexes(&e, &operator)
    }

    fn get_all_deployed_indexes(e: Env) -> Vec<Address> {
        get_all_deployed_indexes(&e)
    }

    fn get_index_count(e: Env, operator: Address) -> u32 {
        let indexes = get_deployed_indexes(&e, &operator);
        indexes.len()
    }

    fn get_total_index_count(e: Env) -> u32 {
        let all_indexes = get_all_deployed_indexes(&e);
        all_indexes.len()
    }

    //   ________  _______  ___________  ___________  _______   _______    ________
    //  /"       )/"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (:   \___/(: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \___  \   \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //   __/  \\  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    //  /" \   :)(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    // (_______/  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    // set_index_contract_wasm
    // Updates the WASM hash for the Index Fund contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - index_contract_wasm: The new WASM hash (BytesN<32>) for the Index Fund contract.
    fn set_index_contract_wasm(e: Env, admin: Address, index_contract_wasm: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let old_wasm = get_index_contract_wasm(&e);
        set_index_contract_wasm(&e, &index_contract_wasm);

        let current_time = e.ledger().timestamp();
        Events::new(&e).index_wasm_updated(
            current_time,
            admin.clone(),
            old_wasm.clone(),
            index_contract_wasm.clone(),
            1,
        );
    }
}

#[contractimpl]
impl UpgradeableContract for IndexFundFactory {
    // version
    // Returns the current version number of the contract.
    //
    // Returns:
    //   - A u32 representing the version.
    fn version() -> u32 {
        100
    }

    // Get contract type symbolic name
    fn contract_name(e: Env) -> Symbol {
        Symbol::new(&e, "IndexFundFactory")
    }

    // commit_upgrade
    // Commits a new WASM hash as a pending upgrade.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - new_wasm_hash: The new WASM hash (BytesN<32>) to be committed.
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
    fn set_emergency_mode(e: Env, emergency_admin: Address, value: bool) {
        emergency_admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&emergency_admin, &Role::EmergencyAdmin);
        set_emergency_mode(&e, &value);

        let current_time = e.ledger().timestamp();
        // Emit factory pause/unpause events based on emergency mode
        if value {
            Events::new(&e).factory_paused(current_time, emergency_admin.clone());
        } else {
            Events::new(&e).factory_unpaused(current_time, emergency_admin.clone());
        }

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
    fn get_emergency_mode(e: Env) -> bool {
        get_emergency_mode(&e)
    }
}

#[contractimpl]
impl TransferableContract for IndexFundFactory {
    // commit_transfer_ownership
    // Commits to transferring ownership of a given role.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - role_name: The symbol representing the role (e.g., "Admin" or "EmergencyAdmin").
    //   - new_address: The new address to assume the role.
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
    fn apply_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name.clone());
        let old_address = access_control.get_role(&role);
        let new_address = access_control.apply_transfer_ownership(&role);

        // Emit factory admin updated event if this is an Admin role transfer
        if role_name == Symbol::new(&e, "Admin") {
            let current_time = e.ledger().timestamp();
            Events::new(&e).factory_admin_updated(current_time, old_address, new_address.clone());
        }

        AccessControlEvents::new(&e).apply_transfer_ownership(role, new_address);
    }

    // revert_transfer_ownership
    // Reverts a pending ownership transfer for a role.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - role_name: The symbol representing the role.
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
