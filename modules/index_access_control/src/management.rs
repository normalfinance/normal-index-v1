use crate::access::IndexAccessControl;
use crate::errors::IndexAccessControlError;
use crate::role::Role;
use crate::storage::StorageTrait;
use soroban_sdk::{panic_with_error, Address, Vec};
use utils::bump::bump_instance;

pub trait SingleAddressManagementTrait {
    fn get_role_safe(&self, role: &Role) -> Option<Address>;
    fn get_role(&self, role: &Role) -> Address;
    fn set_role_address(&self, role: &Role, address: &Address);
}

pub trait MultipleAddressesManagementTrait {
    fn get_role_addresses(&self, role: &Role) -> Vec<Address>;
    fn set_role_addresses(&self, role: &Role, addresses: &Vec<Address>);
}

pub trait MapAddressesManagementTrait {
    fn get_role_address_status_safe(&self, role: &Role, address: &Address) -> Option<bool>;
    fn set_role_address_status(&self, role: &Role, address: &Address, status: bool);
}

impl SingleAddressManagementTrait for IndexAccessControl {
    fn get_role_safe(&self, role: &Role) -> Option<Address> {
        if role.has_many_users() {
            panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage);
        }

        let key = self.get_key(role);
        bump_instance(&self.0);
        self.0.storage().instance().get(&key)
    }

    fn get_role(&self, role: &Role) -> Address {
        match role {
            Role::Admin => {}
            _ => {
                // only admin is guaranteed, use `get_role_safe` for other roles
                panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage);
            }
        }

        match self.get_role_safe(role) {
            Some(address) => address,
            None => panic_with_error!(&self.0, IndexAccessControlError::RoleNotFound),
        }
    }

    fn set_role_address(&self, role: &Role, address: &Address) {
        if role.has_many_users() {
            panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage);
        }

        // require delay if address is being replaced.
        // don't require delay if role is being set for the first time
        let addr = self.get_role_safe(role);
        if addr.is_some() && role.is_transfer_delayed() {
            panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage);
        }

        let key = self.get_key(role);
        bump_instance(&self.0);
        self.0.storage().instance().set(&key, address);
    }
}

impl MultipleAddressesManagementTrait for IndexAccessControl {
    fn get_role_addresses(&self, role: &Role) -> Vec<Address> {
        if !role.has_many_users() {
            panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage);
        }

        let key = self.get_key(role);
        bump_instance(&self.0);
        let addresses: Vec<Address> = self
            .0
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(Vec::new(&self.0));

        addresses
    }

    // no delay-related code as we require it only for single addresses roles
    fn set_role_addresses(&self, role: &Role, addresses: &Vec<Address>) {
        if !role.has_many_users() || role.is_transfer_delayed() {
            panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage);
        }

        let key = self.get_key(role);
        bump_instance(&self.0);
        let old_addresses = self.get_role_addresses(role);
        self.0.storage().instance().set(&key, addresses);

        let old_len = old_addresses.len();
        for i in 0..old_len {
            let old_address = old_addresses.get_unchecked(i);
            self.set_role_address_status(role, &old_address, false);
        }

        let len = addresses.len();
        for i in 0..len {
            let address = addresses.get_unchecked(i);
            self.set_role_address_status(role, &address, true);
        }
    }
}

impl MapAddressesManagementTrait for IndexAccessControl {
    fn get_role_address_status_safe(&self, role: &Role, address: &Address) -> Option<bool> {
        if !role.has_many_users() {
            panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage);
        }

        let key = self.get_address_key(role, address);
        bump_instance(&self.0);
        self.0.storage().instance().get(&key)
    }

    fn set_role_address_status(&self, role: &Role, address: &Address, status: bool) {
        if !role.has_many_users() || role.is_transfer_delayed() {
            panic_with_error!(&self.0, IndexAccessControlError::BadRoleUsage);
        }

        let key = self.get_address_key(role, address);
        bump_instance(&self.0);
        self.0.storage().instance().set(&key, &status);

        let mut addresses = self.get_role_addresses(role);
        let len = addresses.len();
        let mut contains = false;
        for i in 0..len {
            if addresses.get_unchecked(i) == *address {
                contains = true;
                break;
            }
        }

        if status && !contains {
            addresses.push_back(address.clone());
            let vec_key = self.get_key(role);
            self.0.storage().instance().set(&vec_key, &addresses);
            return;
        }

        if !status && contains {
            let mut updated = Vec::new(&self.0);
            for i in 0..len {
                let current = addresses.get_unchecked(i);
                if current != *address {
                    updated.push_back(current);
                }
            }
            let vec_key = self.get_key(role);
            self.0.storage().instance().set(&vec_key, &updated);
        }
    }
}
