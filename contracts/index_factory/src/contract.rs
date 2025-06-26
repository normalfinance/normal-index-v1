use crate::events::{ Events, FactoryConfigEvents, FactoryEvents };
use crate::storage::{
    get_aggregator, get_contract_sequence, get_fee_contract_wasm, set_aggregator, set_contract_sequence, set_fee_contract_wasm, set_max_manager_fee_fraction, set_protocol_fee_fraction, DexDistribution
};
use access_control::access::{ AccessControl, AccessControlTrait };
use access_control::emergency::{ get_emergency_mode, set_emergency_mode };
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::{ Role, SymbolRepresentation };
use access_control::transfer::TransferOwnershipTrait;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, Bytes, BytesN, Env, IntoVal, Symbol, Vec
};
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{ apply_upgrade, commit_upgrade, revert_upgrade };

#[contract]
pub struct IndexFactory;

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
    //   - index_contract_wasm: The WASM hash (BytesN<32>) for the swap fee contract.
    pub fn __constructor(
        e: Env,
        admin: Address,
        emergency_admin: Address,
        aggregator: Address,
        index_contract_wasm: BytesN<32>,
        max_manager_fee_fraction: u32,
        protocol_fee_fraction: u32
    ) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);
        access_control.commit_transfer_ownership(&Role::EmergencyAdmin, &emergency_admin);
        access_control.apply_transfer_ownership(&Role::EmergencyAdmin);

        set_aggregator(&e, &aggregator);
        set_fee_contract_wasm(&e, &index_contract_wasm);
        set_protocol_fee_fraction(&e, &protocol_fee_fraction);
        set_max_manager_fee_fraction(&e, &max_manager_fee_fraction);
    }

    // set_index_contract_wasm
    // Updates the WASM hash for the swap fee contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - index_contract_wasm: The new WASM hash (BytesN<32>) for the swap fee contract.
    pub fn set_index_contract_wasm(e: Env, admin: Address, index_contract_wasm: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        set_fee_contract_wasm(&e, &index_contract_wasm);
        Events::new(&e).set_wasm(index_contract_wasm);
    }

    // set_index_contract_wasm
    // Updates the WASM hash for the swap fee contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - fraction: The new WASM hash (BytesN<32>) for the swap fee contract.
    pub fn set_protocol_fee_fraction(e: Env, admin: Address, fraction: u32) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        set_protocol_fee_fraction(&e, &fraction);
    }

    // set_index_contract_wasm
    // Updates the WASM hash for the swap fee contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - fraction: The new WASM hash (u32) for the swap fee contract.
    pub fn set_max_manager_fee_fraction(e: Env, admin: Address, fraction: u32) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        set_max_manager_fee_fraction(&e, &fraction);
    }

    // deploy_index_contract
    // Deploys a new swap fee contract instance.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - operator: The address of the operator (must be authorized).
    //   - fee_destination: The address where fees are sent.
    //   - max_swap_fee_fraction: The fee in basis points to be set in the new contract.
    //
    // Returns:
    //   - The address of the newly deployed swap fee contract.
    pub fn deploy_index_contract(
        e: Env,
        operator: Address,
        fee_destination: Address,
        max_max_swap_fee_fraction: u32
    ) -> Address {
        operator.require_auth();

        let sequence = get_contract_sequence(&e, operator.clone());
        set_contract_sequence(&e, operator.clone(), sequence + 1);
        let mut salt = Bytes::new(&e);
        salt.append(&operator.clone().to_xdr(&e));
        salt.append(&sequence.to_xdr(&e));
        let address = e
            .deployer()
            .with_current_contract(e.crypto().sha256(&salt))
            .deploy_v2(get_fee_contract_wasm(&e), (
                get_router(&e),
                operator.clone(),
                fee_destination.clone(),
                max_max_swap_fee_fraction,
            ));
        Events::new(&e).deploy(
            operator,
            fee_destination,
            max_max_swap_fee_fraction,
            address.clone()
        );
        address
    }

    pub fn swap(
        e: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        amount_out_min: i128,
        distribution: Vec<DexDistribution>,
        to: Address,
        deadline: u64
    ) -> Vec<Vec<i128>> {
        let result: Vec<Vec<i128>> = e.invoke_contract(
            &get_aggregator(&e),
            &symbol_short!("swap_exact_tokens_for_tokens"),
            Vec::from_array(&e, [
                e.current_contract_address().into_val(&e),
                token_in.into_val(&e),
                token_out.into_val(&e),
                amount_in.into_val(&e),
                amount_out_min.into_val(&e),
                distribution.into_val(&e),
                to.into_val(&e),
                deadline.into_val(&e),
            ])
        );

        result
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

        let role = Role::from_symbol(&e, role_name);
        let new_address = access_control.apply_transfer_ownership(&role);
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
            0 =>
                match access_control.get_role_safe(&role) {
                    Some(address) => address,
                    None => panic_with_error!(&e, AccessControlError::RoleNotFound),
                }
            _ => access_control.get_future_address(&role),
        }
    }
}
