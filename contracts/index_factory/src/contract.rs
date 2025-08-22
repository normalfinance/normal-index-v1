use crate::events::Events;
use crate::events::FactoryConfigEvents;
use crate::events::FactoryEvents;
use crate::index_utils::get_index_salt;
use crate::interface::{AdminInterface, IndexFactoryTrait};
use crate::storage::get_index_contract_wasm;
use crate::storage::get_index_fee_enabled;
use crate::storage::get_token_contract_wasm;
use crate::storage::set_index_contract_wasm;
use crate::storage::set_index_fee_enabled;
use crate::storage::set_is_killed_create;
use crate::storage::set_token_contract_wasm;
use crate::storage::{
    add_deployed_index, get_aggregator, get_all_deployed_indexes, get_contract_sequence,
    get_deployed_indexes, get_fee_contract_wasm, get_index_fee_enabled,
    get_max_manager_fee_fraction, get_minimum_fee_threshold, get_protocol_fee_fraction,
    get_protocol_fee_recipient, get_router, set_aggregator, set_contract_sequence,
    set_fee_contract_wasm, set_index_fee_enabled, set_max_manager_fee_fraction,
    set_minimum_fee_threshold, set_protocol_fee_fraction, set_protocol_fee_recipient, set_router,
    DexDistribution,
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
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, symbol_short, Address, BytesN, Env,
    IntoVal, Symbol, Vec,
};
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};
use utils::storage::IndexParams;

#[contract]
pub struct IndexFactory;

// Factory configuration struct for query methods
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactoryConfig {
    pub aggregator: Address,
    pub router: Address,
    pub protocol_fee_fraction: u32,
    pub max_manager_fee_fraction: u32,
    pub protocol_fee_recipient: Address,
    pub minimum_fee_threshold: u128,
    pub index_contract_wasm: BytesN<32>,
    pub token_contract_wasm: BytesN<32>,
}

#[contractimpl]
impl IndexFactory {
    // __constructor
    // Initializes the factory by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The address to be assigned the Admin role.
    //   - emergency_admin: The address to be assigned the EmergencyAdmin role.
    //   - aggregator: The address of the swap aggregator contract.
    //   - router: The address of the router contract.
    //   - index_contract_wasm: The WASM hash (BytesN<32>) for the swap fee contract.
    pub fn __constructor(
        e: Env,
        admin: Address,
        emergency_admin: Address,
        aggregator: Address,
        router: Address,
        index_contract_wasm: BytesN<32>,
        token_contract_wasm: BytesN<32>,
        max_manager_fee_fraction: u32,
        protocol_fee_fraction: u32,
        protocol_fee_recipient: Address,
        minimum_fee_threshold: u128,
    ) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);
        access_control.commit_transfer_ownership(&Role::EmergencyAdmin, &emergency_admin);
        access_control.apply_transfer_ownership(&Role::EmergencyAdmin);

        set_aggregator(&e, &aggregator);
        set_router(&e, &router);
        set_index_contract_wasm(&e, &index_contract_wasm);
        set_token_contract_wasm(&e, &token_contract_wasm);
        set_protocol_fee_fraction(&e, &protocol_fee_fraction);
        set_max_manager_fee_fraction(&e, &max_manager_fee_fraction);
        set_protocol_fee_recipient(&e, &protocol_fee_recipient);
        // Set universal minimum fee threshold - IMMUTABLE after initialization
        set_minimum_fee_threshold(&e, &minimum_fee_threshold);
    }
}

impl IndexFactoryTrait for IndexFactory {
    // deploy_index_contract
    // Deploys a new swap fee contract instance.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - params: The address where fees are sent (params.admin must be authorized).
    //
    // Returns:
    //   - The address of the newly deployed swap fee contract.
    fn deploy_index_contract(e: Env, params: IndexParams) -> Address {
        params.admin.require_auth();

        let sequence = get_contract_sequence(&e, params.admin.clone());
        set_contract_sequence(&e, params.admin.clone(), sequence + 1);

        let salt = get_index_salt(&e, &params.admin, &sequence);

        let address = e.deployer().with_current_contract(salt).deploy_v2(
            get_index_contract_wasm(&e),
            (get_router(&e), get_token_contract_wasm(&e), params.clone()),
        );

        // Add to index registry
        add_deployed_index(&e, &params.admin, &address);

        // Emit enhanced deployment event
        let current_time = e.ledger().timestamp();
        let initial_components = Vec::new(&e); // Empty initially
        let initial_weights = Vec::new(&e); // Empty initially
        let base_nav = 0; // TODO: Get from contract parameters
        let initial_price = 0; // TODO: Get from contract parameters
        let is_public = false; // TODO: Get from contract parameters
        let deployment_cost = 0; // TODO: Calculate actual deployment cost

        // TODO: fix correctly
        Events::new(&e).index_deployed(
            current_time,
            params.admin.clone(),
            address.clone(), // index_address
            params.admin.clone(),
            params.admin.clone(), // manager (using fee_destination as manager for now)
            params.admin.clone(),
            0,
            initial_components,
            initial_weights,
            base_nav,
            initial_price,
            is_public,
            deployment_cost,
        );

        address
    }
}

impl AdminInterface for IndexFactory {
    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    // Gets the protocol fee recipient address.
    fn get_protocol_fee_recipient(e: Env) -> Address {
        get_protocol_fee_recipient(&e)
    }

    // Query Methods - Factory Configuration
    fn get_factory_config(e: Env) -> FactoryConfig {
        FactoryConfig {
            aggregator: get_aggregator(&e),
            router: get_router(&e),
            protocol_fee_fraction: get_protocol_fee_fraction(&e),
            max_manager_fee_fraction: get_max_manager_fee_fraction(&e),
            protocol_fee_recipient: get_protocol_fee_recipient(&e),
            minimum_fee_threshold: get_minimum_fee_threshold(&e),
            index_contract_wasm: get_index_contract_wasm(&e),
            token_contract_wasm: get_token_contract_wasm(&e),
        }
    }

    // Individual getters for factory configuration
    fn get_aggregator(e: Env) -> Address {
        get_aggregator(&e)
    }

    fn get_router(e: Env) -> Address {
        get_router(&e)
    }

    fn get_protocol_fee_fraction(e: Env) -> u32 {
        get_protocol_fee_fraction(&e)
    }

    fn get_index_fee_enabled(e: Env, index_address: Address) -> bool {
        get_index_fee_enabled(&e, &index_address)
    }

    fn get_max_manager_fee_fraction(e: Env) -> u32 {
        get_max_manager_fee_fraction(&e)
    }

    fn get_minimum_fee_threshold(e: Env) -> u128 {
        get_minimum_fee_threshold(&e)
    }

    fn get_index_contract_wasm(e: Env) -> BytesN<32> {
        get_index_contract_wasm(&e)
    }

    fn get_token_contract_wasm(e: Env) -> BytesN<32> {
        get_token_contract_wasm(&e)
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
    // Updates the WASM hash for the swap fee contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - index_contract_wasm: The new WASM hash (BytesN<32>) for the swap fee contract.
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

    fn set_token_contract_wasm(e: Env, admin: Address, token_contract_wasm: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let old_wasm = get_token_contract_wasm(&e);
        set_token_contract_wasm(&e, &token_contract_wasm);

        let current_time = e.ledger().timestamp();
        Events::new(&e).token_wasm_updated(
            current_time,
            admin.clone(),
            old_wasm.clone(),
            token_contract_wasm.clone(),
            1,
        );
    }

    fn set_protocol_fee_fraction(e: Env, admin: Address, fraction: u32) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let old_fee = get_protocol_fee_fraction(&e);
        set_protocol_fee_fraction(&e, &fraction);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).protocol_fee_updated(current_time, admin, old_fee, fraction);
    }

    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        set_protocol_fee_recipient(&e, &recipient);
    }

    // set_max_manager_fee_fraction
    // .
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - fraction: The new WASM hash (u32) for the swap fee contract.
    fn set_max_manager_fee_fraction(e: Env, admin: Address, fraction: u32) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let old_max_fee = get_max_manager_fee_fraction(&e);
        set_max_manager_fee_fraction(&e, &fraction);

        let current_time = e.ledger().timestamp();
        // Emit enhanced event
        Events::new(&e).max_management_fee_updated(current_time, admin, old_max_fee, fraction);
    }

    // set_minimum_fee_threshold
    // Updates the universal minimum fee threshold for all indexes.
    // Only the protocol admin can call this function.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - threshold: The new minimum fee threshold (u128) in token units.
    fn set_minimum_fee_threshold(e: Env, admin: Address, threshold: u128) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        set_minimum_fee_threshold(&e, &threshold);
    }

    // set_index_fee_enabled
    // Toggle fee collection for a specific index.
    // Only the factory admin can call this function.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - index_address: The address of the index contract.
    //   - enabled: Whether to enable (true) or disable (false) fees for this index.
    fn set_index_fee_enabled(e: Env, admin: Address, index_address: Address, enabled: bool) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let old_status = get_index_fee_enabled(&e, &index_address);
        set_index_fee_enabled(&e, &index_address, enabled);

        // Emit event if status changed
        if old_status != enabled {
            Events::new(&e).index_fee_toggled(index_address, enabled);
        }
    }

    // batch_set_index_fee_enabled
    // Toggle fee collection for multiple indexes at once.
    // Only the factory admin can call this function.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - index_settings: Vec of (index_address, enabled) pairs.
    fn batch_set_index_fee_enabled(e: Env, admin: Address, index_settings: Vec<(Address, bool)>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        for setting in index_settings.iter() {
            let (index_address, enabled) = setting;
            let old_status = get_index_fee_enabled(&e, &index_address);
            set_index_fee_enabled(&e, &index_address, enabled);

            // Emit event if status changed
            if old_status != enabled {
                Events::new(&e).index_fee_toggled(index_address, enabled);
            }
        }
    }

    //    _______     __       ____  ____   ________  _______  ________
    //   |   __ "\   /""\     ("  _||_ " | /"       )/"     "||"      "\
    //   (. |__) :) /    \    |   (  ) : |(:   \___/(: ______)(.  ___  :)
    //   |:  ____/ /' /\  \   (:  |  | . ) \___  \   \/    |  |: \   ) ||
    //   (|  /    //  __'  \   \\ \__/ //   __/  \\  // ___)_ (| (___\ ||
    //  /|__/ \  /   /  \\  \  /\\ __ //\  /" \   :)(:      "||:       :)
    // (_______)(___/    \___)(__________)(_______/  \_______)(________/

    fn kill_create(e: Env, admin: Address) {
        admin.require_auth();
        require_admin(&e, &admin);

        set_is_killed_create(&e, &true);
        // FactoryEvents::new(&e).kill_create();
    }

    fn unkill_create(e: Env, admin: Address) {
        admin.require_auth();
        require_admin(&e, &admin);

        set_is_killed_create(&e, &false);
        // FactoryEvents::new(&e).unkill_create();
    }

    fn get_is_killed_create(e: Env) -> bool {
        return false;
        // get_is_killed_create(&e)
    }
}

#[contractimpl]
impl UpgradeableContract for IndexFactory {
    // version
    // Returns the current version number of the contract.
    //
    // Returns:
    //   - A u32 representing the version.
    fn version() -> u32 {
        150
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
impl TransferableContract for IndexFactory {
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
