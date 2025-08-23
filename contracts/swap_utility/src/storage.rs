use soroban_sdk::{contracttype, Address, Env};
use utils::bump::{bump_instance, bump_persistent};

use crate::interface::{DexProvider, ProviderConfig};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    // Configuration keys
    ProviderConfig(DexProvider),
    AdminAddress,
    Initialized,

    // Provider-specific settings
    DefaultProvider,

    // Asset addresses
    XlmTokenAddress,
}

// Admin and initialization functions
pub fn set_admin(env: &Env, admin: &Address) {
    bump_instance(env);
    env.storage().instance().set(&DataKey::AdminAddress, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    bump_instance(env);
    env.storage().instance().get(&DataKey::AdminAddress)
}

pub fn require_admin(env: &Env, address: &Address) {
    match get_admin(env) {
        Some(admin) => {
            if admin != *address {
                panic!("Unauthorized: only admin can perform this action");
            }
        }
        None => panic!("Contract not initialized"),
    }
}

pub fn set_initialized(env: &Env) {
    bump_instance(env);
    env.storage().instance().set(&DataKey::Initialized, &true);
}

pub fn is_initialized(env: &Env) -> bool {
    bump_instance(env);
    env.storage()
        .instance()
        .get(&DataKey::Initialized)
        .unwrap_or(false)
}

// Provider configuration functions
pub fn set_provider_config(env: &Env, provider: DexProvider, config: &ProviderConfig) {
    let key = DataKey::ProviderConfig(provider);
    bump_persistent(env, &key);
    env.storage().persistent().set(&key, config);
}

pub fn get_provider_config(env: &Env, provider: DexProvider) -> Option<ProviderConfig> {
    let key = DataKey::ProviderConfig(provider);
    match env.storage().persistent().get(&key) {
        Some(config) => {
            bump_persistent(env, &key);
            Some(config)
        }
        None => None,
    }
}

pub fn is_provider_configured(env: &Env, provider: DexProvider) -> bool {
    get_provider_config(env, provider).is_some()
}

pub fn is_provider_active(env: &Env, provider: DexProvider) -> bool {
    match get_provider_config(env, provider) {
        Some(config) => config.is_active,
        None => false,
    }
}

// Default provider management
pub fn set_default_provider(env: &Env, provider: DexProvider) {
    bump_instance(env);
    env.storage()
        .instance()
        .set(&DataKey::DefaultProvider, &provider);
}

pub fn get_default_provider(env: &Env) -> DexProvider {
    bump_instance(env);
    env.storage()
        .instance()
        .get(&DataKey::DefaultProvider)
        .unwrap_or_default()
}

// Utility functions
pub fn require_initialized(env: &Env) {
    if !is_initialized(env) {
        panic!("Contract not initialized");
    }
}

pub fn require_provider_configured(env: &Env, provider: DexProvider) {
    if !is_provider_configured(env, provider) {
        panic!("Provider not configured");
    }
}

pub fn require_provider_active(env: &Env, provider: DexProvider) {
    if !is_provider_active(env, provider) {
        panic!("Provider not active");
    }
}

// XLM token address management
pub fn set_xlm_token_address(env: &Env, xlm_address: &Address) {
    bump_instance(env);
    env.storage()
        .instance()
        .set(&DataKey::XlmTokenAddress, xlm_address);
}

pub fn get_xlm_token_address(env: &Env) -> Option<Address> {
    bump_instance(env);
    env.storage().instance().get(&DataKey::XlmTokenAddress)
}
