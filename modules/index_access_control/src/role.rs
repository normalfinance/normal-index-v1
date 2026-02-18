use crate::errors::IndexAccessControlError;
use soroban_sdk::{panic_with_error, Env, Symbol};

#[derive(Clone)]
pub enum Role {
    Admin,
    EmergencyAdmin,
    FeeAdmin,
    RewardsAdmin,
    OperationsAdmin,
    RebalanceAuthorities,
}

impl Role {
    pub(crate) fn has_many_users(&self) -> bool {
        match self {
            Role::Admin => false,
            Role::EmergencyAdmin => false,
            Role::FeeAdmin => false,
            Role::RewardsAdmin => false,
            Role::OperationsAdmin => false,
            Role::RebalanceAuthorities => true,
        }
    }

    pub(crate) fn is_transfer_delayed(&self) -> bool {
        match self {
            Role::Admin => true,
            Role::EmergencyAdmin => true,
            Role::FeeAdmin => false,
            Role::RewardsAdmin => false,
            Role::OperationsAdmin => false,
            Role::RebalanceAuthorities => false,
        }
    }
}

pub trait SymbolRepresentation {
    fn as_symbol(&self, e: &Env) -> Symbol;
    fn from_symbol(e: &Env, value: Symbol) -> Self;
}

impl SymbolRepresentation for Role {
    fn as_symbol(&self, e: &Env) -> Symbol {
        match self {
            Role::Admin => Symbol::new(&e, "Admin"),
            Role::EmergencyAdmin => Symbol::new(&e, "EmergencyAdmin"),
            Role::FeeAdmin => Symbol::new(&e, "FeeAdmin"),
            Role::RewardsAdmin => Symbol::new(&e, "RewardsAdmin"),
            Role::OperationsAdmin => Symbol::new(&e, "OperationsAdmin"),
            Role::RebalanceAuthorities => Symbol::new(&e, "RebalanceAuthorities"),
        }
    }

    fn from_symbol(e: &Env, value: Symbol) -> Self {
        if value == Symbol::new(e, "Admin") {
            return Role::Admin;
        } else if value == Symbol::new(e, "EmergencyAdmin") {
            return Role::EmergencyAdmin;
        } else if value == Symbol::new(e, "RewardsAdmin") {
            return Role::RewardsAdmin;
        } else if value == Symbol::new(e, "FeeAdmin") {
            return Role::FeeAdmin;
        } else if value == Symbol::new(e, "OperationsAdmin") {
            return Role::OperationsAdmin;
        } else if value == Symbol::new(e, "RebalanceAuthorities") {
            return Role::RebalanceAuthorities;
        }
        panic_with_error!(e, IndexAccessControlError::BadRoleUsage);
    }
}
