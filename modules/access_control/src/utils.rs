use crate::access::{AccessControl, AccessControlTrait};
use crate::errors::AccessControlError;
use crate::role::Role;
use soroban_sdk::{panic_with_error, Address, Env};

pub fn require_admin(e: &Env, address: &Address) {
    let access_control = AccessControl::new(e);
    let _ = access_control.address_has_role(address, &Role::Admin)
        || panic_with_error!(e, AccessControlError::Unauthorized);
}

pub fn require_rewards_admin_or_owner(e: &Env, address: &Address) {
    let access_control = AccessControl::new(e);
    let _ = access_control.address_has_role(address, &Role::Admin)
        || access_control.address_has_role(address, &Role::RewardsAdmin)
        || panic_with_error!(e, AccessControlError::Unauthorized);
}

pub fn require_operations_admin_or_owner(e: &Env, address: &Address) {
    let access_control = AccessControl::new(e);
    let _ = access_control.address_has_role(address, &Role::OperationsAdmin)
        || access_control.address_has_role(address, &Role::Admin)
        || panic_with_error!(e, AccessControlError::Unauthorized);
}

pub fn require_fee_admin_or_owner(e: &Env, address: &Address) {
    let access_control = AccessControl::new(e);
    let _ = access_control.address_has_role(address, &Role::FeeAdmin)
        || access_control.address_has_role(address, &Role::Admin)
        || panic_with_error!(e, AccessControlError::Unauthorized);
}
