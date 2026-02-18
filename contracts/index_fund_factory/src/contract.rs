use crate::events::Events;
use crate::events::FactoryConfigEvents;
use crate::events::FactoryEvents;
use crate::index_utils::get_index_salt;
use crate::interface::{AdminInterface, IndexFundFactoryTrait};
use crate::storage::FactoryConfig;
use soroban_sdk::Bytes;
use soroban_sdk::Map;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, BytesN, Env, IntoVal, Symbol, Vec,
};

// Types
use types::component::RebalanceParams;
use types::component::RefactorParams;
use types::index::IndexParams;

// Access control
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::{Role, SymbolRepresentation};
use access_control::transfer::TransferOwnershipTrait;

// Upgrade
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};

#[contract]
pub struct IndexFundFactory;

#[contractimpl]
impl IndexFundFactory {
    // __constructor
    // Initializes the factory by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The address to be assigned the Admin role.
    //   - emergency_admin: The address to be assigned the EmergencyAdmin role.
    //   - rew
    //   - index_contract_wasm: The WASM hash (BytesN<32>) for the swap fee contract.
    pub fn __constructor(
        e: Env,
        admin: Address,
        emergency_admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        fee_admin: Address,
        index_contract_wasm: BytesN<32>,
    ) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &emergency_admin);
        access_control.set_role_address(&Role::RewardsAdmin, &rewards_admin);
        access_control.set_role_address(&Role::OperationsAdmin, &operations_admin);
        access_control.set_role_address(&Role::FeeAdmin, &fee_admin);

        crate::storage::set_index_contract_wasm(&e, &index_contract_wasm);
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

        let sequence = crate::storage::get_contract_sequence(&e, params.admin.clone());
        crate::storage::set_contract_sequence(&e, params.admin.clone(), sequence + 1);

        let salt = get_index_salt(&e, &params.admin, &sequence);

        let address = e.deployer().with_current_contract(salt).deploy_v2(
            crate::storage::get_index_contract_wasm(&e),
            (
                e.current_contract_address(),
                serialized_asset.clone(),
                params.clone(),
            ),
        );

        // Add to index registry
        crate::storage::add_deployed_index(&e, &params.admin, &address);

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

    fn mint(e: Env, user: Address, index: Address, amount: u128) {
        user.require_auth();
        e.invoke_contract::<()>(
            &index,
            &Symbol::new(&e, "mint"),
            Vec::from_array(&e, [user.clone().into_val(&e), amount.into_val(&e)]),
        );
        Events::new(&e).index_mint(e.ledger().timestamp(), index, user, amount);
    }

    fn redeem(e: Env, user: Address, index: Address, share_amount: u128) {
        user.require_auth();
        e.invoke_contract::<()>(
            &index,
            &Symbol::new(&e, "redeem"),
            Vec::from_array(&e, [user.clone().into_val(&e), share_amount.into_val(&e)]),
        );
        Events::new(&e).index_redeem(e.ledger().timestamp(), index, user, share_amount);
    }

    fn rebalance(e: Env, caller: Address, index: Address, params: RebalanceParams) {
        caller.require_auth();
        e.invoke_contract::<()>(
            &index,
            &Symbol::new(&e, "rebalance"),
            Vec::from_array(&e, [caller.clone().into_val(&e), params.into_val(&e)]),
        );
        Events::new(&e).index_rebalance(e.ledger().timestamp(), index, caller);
    }

    fn refactor(e: Env, caller: Address, index: Address, params: RefactorParams) {
        caller.require_auth();
        e.invoke_contract::<()>(
            &index,
            &Symbol::new(&e, "refactor"),
            Vec::from_array(&e, [caller.clone().into_val(&e), params.into_val(&e)]),
        );
        Events::new(&e).index_refactor(e.ledger().timestamp(), index, caller);
    }

    fn claim_system_fees(
        e: Env,
        caller: Address,
        index: Address,
        token: Address,
        destination: Address,
    ) -> u128 {
        caller.require_auth();
        let amount = e.invoke_contract::<u128>(
            &index,
            &Symbol::new(&e, "claim_system_fees"),
            Vec::from_array(
                &e,
                [
                    caller.clone().into_val(&e),
                    token.clone().into_val(&e),
                    destination.clone().into_val(&e),
                ],
            ),
        );
        Events::new(&e).index_claim_system_fees(
            e.ledger().timestamp(),
            index,
            caller,
            token,
            amount,
            destination,
        );
        amount
    }

    fn claim_manager_fees(
        e: Env,
        caller: Address,
        index: Address,
        token: Address,
        destination: Address,
    ) -> u128 {
        caller.require_auth();
        let amount = e.invoke_contract::<u128>(
            &index,
            &Symbol::new(&e, "claim_manager_fees"),
            Vec::from_array(
                &e,
                [
                    caller.clone().into_val(&e),
                    token.clone().into_val(&e),
                    destination.clone().into_val(&e),
                ],
            ),
        );
        Events::new(&e).index_claim_manager_fees(
            e.ledger().timestamp(),
            index,
            caller,
            token,
            amount,
            destination,
        );
        amount
    }
}

#[contractimpl]
impl AdminInterface for IndexFundFactory {
    // Sets the privileged addresses.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `rewards_admin` - The address of the rewards admin.
    // * `operations_admin` - The address of the operations admin.
    // * `pause_admin` - The address of the pause admin.
    // * `emergency_pause_admin` - The addresses of the emergency pause admins.
    // * `system_fee_admin` - The address of the system fee admin.
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

        result
    }

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
            index_contract_wasm: crate::storage::get_index_contract_wasm(&e),
        }
    }

    fn get_index_contract_wasm(e: Env) -> BytesN<32> {
        crate::storage::get_index_contract_wasm(&e)
    }

    // Index Registry Query Methods
    fn get_deployed_indexes(e: Env, operator: Address) -> Vec<Address> {
        crate::storage::get_deployed_indexes(&e, &operator)
    }

    fn get_all_deployed_indexes(e: Env) -> Vec<Address> {
        crate::storage::get_all_deployed_indexes(&e)
    }

    fn get_index_count(e: Env, operator: Address) -> u32 {
        let indexes = crate::storage::get_deployed_indexes(&e, &operator);
        indexes.len()
    }

    fn get_total_index_count(e: Env) -> u32 {
        let all_indexes = crate::storage::get_all_deployed_indexes(&e);
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

        let old_wasm = crate::storage::get_index_contract_wasm(&e);
        crate::storage::set_index_contract_wasm(&e, &index_contract_wasm);

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
