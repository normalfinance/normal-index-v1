use crate::access::IndexAccessControl;
use crate::errors::IndexAccessControlError;
use crate::role::Role;
use soroban_sdk::{contracttype, panic_with_error, Address};

/// Instance storage keys used by index access-control role management.
#[derive(Clone)]
#[contracttype]
pub(crate) enum DataKey {
    /// Owner role allowed to manage privileged roles and upgrades.
    Admin,
    /// Emergency admin allowed to toggle emergency mode and emergency upgrade paths.
    EmergencyAdmin,
    /// Legacy operator role (rewards admin).
    Operator,
    /// Operations admin role for day-to-day protocol operations.
    OperationsAdmin,
    /// Fee admin role for fee-related configuration.
    FeeAdmin,
    /// Vector containing all rebalance authority addresses.
    RebalanceAuthoritiesVec,
    /// Boolean key for a specific rebalance authority address.
    RebalanceAuthorities(Address),

    /// Pending new admin for two-step transfers.
    FutureAdmin,
    /// Pending new emergency admin for two-step transfers.
    FutureEmergencyAdmin,

    /// Deadline for completing admin ownership transfer.
    TransferOwnershipDeadline,
    /// Deadline for completing emergency-admin ownership transfer.
    EmAdminTransferOwnershipDeadline,

    /// Emergency-mode status flag.
    EmergencyMode,
}

/// Maps index access-control roles to concrete storage keys.
pub(crate) trait StorageTrait {
    /// Returns the primary key for a role.
    /// Arguments: `role` (`&Role`). Returns: `DataKey`.
    fn get_key(&self, role: &Role) -> DataKey;
    /// Returns the address-specific key for roles with per-address entries.
    /// Arguments: `role` (`&Role`), `address` (`&Address`). Returns: `DataKey`.
    fn get_address_key(&self, role: &Role, address: &Address) -> DataKey;
    /// Returns the pending-transfer key for a role.
    /// Arguments: `role` (`&Role`). Returns: `DataKey`.
    fn get_future_key(&self, role: &Role) -> DataKey;
    /// Returns the transfer deadline key for a role.
    /// Arguments: `role` (`&Role`). Returns: `DataKey`.
    fn get_future_deadline_key(&self, role: &Role) -> DataKey;
}

impl StorageTrait for IndexAccessControl {
    /// Maps a role to its primary storage key.
    /// Arguments: `role` (`&Role`). Returns: `DataKey`.
    fn get_key(&self, role: &Role) -> DataKey {
        match role {
            Role::Admin => DataKey::Admin,
            Role::EmergencyAdmin => DataKey::EmergencyAdmin,
            Role::FeeAdmin => DataKey::FeeAdmin,
            Role::RewardsAdmin => DataKey::Operator,
            Role::OperationsAdmin => DataKey::OperationsAdmin,
            Role::RebalanceAuthorities => DataKey::RebalanceAuthoritiesVec,
        }
    }

    /// Maps a role/address pair to its per-address storage key.
    /// Arguments: `role` (`&Role`), `address` (`&Address`). Returns: `DataKey`.
    fn get_address_key(&self, role: &Role, address: &Address) -> DataKey {
        match role {
            Role::RebalanceAuthorities => DataKey::RebalanceAuthorities(address.clone()),
            _ => panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage),
        }
    }

    /// Maps a role to its pending-transfer key.
    /// Arguments: `role` (`&Role`). Returns: `DataKey`.
    fn get_future_key(&self, role: &Role) -> DataKey {
        match role {
            Role::Admin => DataKey::FutureAdmin,
            Role::EmergencyAdmin => DataKey::FutureEmergencyAdmin,
            _ => panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage),
        }
    }

    /// Maps a role to its transfer-deadline key.
    /// Arguments: `role` (`&Role`). Returns: `DataKey`.
    fn get_future_deadline_key(&self, role: &Role) -> DataKey {
        match role {
            Role::Admin => DataKey::TransferOwnershipDeadline,
            Role::EmergencyAdmin => DataKey::EmAdminTransferOwnershipDeadline,
            _ => panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage),
        }
    }
}
