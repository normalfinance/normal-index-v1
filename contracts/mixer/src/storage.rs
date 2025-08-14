use soroban_sdk::{Address, Env, Map, Symbol, Vec};
use utils::bump::{bump_instance, bump_persistent};
use crate::types::{
    IndexCredits, WithdrawalRequest, DepositRecord, MixerConfig, PortfolioTier, AssetAllocation,
};
use crate::errors::MixerError;

const DAY_IN_LEDGERS: u32 = 17280; // Assuming 5-second ledgers
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS * 30; // 30 days
const INSTANCE_BUMP_AMOUNT: u32 = DAY_IN_LEDGERS * 30; // 30 days

#[derive(Clone)]
#[soroban_sdk::contracttype]
pub enum DataKey {
    // Configuration
    Admin,
    Config,
    IsInitialized,
    
    // Index management
    IndexCredits(Address),
    IndexRegistry,
    AuthorizedIndexes,
    
    // Asset tracking
    TotalAssetBalance(Symbol),
    SupportedAssets,
    AssetPrices(Symbol),
    
    // Withdrawal management
    WithdrawalRequest(u32),
    PendingWithdrawals,
    WithdrawalCounter,
    
    // Deposit tracking
    DepositRecord(u32),
    DepositCounter,
    
    // Privacy and anonymity
    AnonymitySet,
    TierDistribution,
    
    // Rate limiting
    WithdrawalRateLimit,
    LastWithdrawalBatch,
}

// Admin functions
pub fn get_admin(e: &Env) -> Address {
    bump_instance(e);
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_admin(e: &Env, admin: &Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn is_initialized(e: &Env) -> bool {
    bump_instance(e);
    e.storage().instance().get(&DataKey::IsInitialized).unwrap_or(false)
}

pub fn set_initialized(e: &Env) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::IsInitialized, &true);
}

// Configuration
pub fn get_mixer_config(e: &Env) -> MixerConfig {
    bump_instance(e);
    e.storage().instance().get(&DataKey::Config).unwrap_or(MixerConfig {
        min_anonymity_set: 5,
        withdrawal_delay: 3600, // 1 hour
        max_withdrawals_per_period: 10,
        rate_limit_period: 86400, // 24 hours
        tier_grace_period: 2592000, // 30 days
        tier_noise_factor: 1000, // 10%
        is_active: true,
    })
}

pub fn set_mixer_config(e: &Env, config: &MixerConfig) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::Config, config);
}

// Index Credits Management
pub fn get_index_credits(e: &Env, index_address: &Address) -> Option<IndexCredits> {
    let key = DataKey::IndexCredits(index_address.clone());
    e.storage().persistent().get(&key).map(|credits| {
        bump_persistent(e, &key);
        credits
    })
}

pub fn set_index_credits(e: &Env, index_address: &Address, credits: &IndexCredits) {
    let key = DataKey::IndexCredits(index_address.clone());
    e.storage().persistent().set(&key, credits);
    e.storage().persistent().extend_ttl(&key, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_index_credits_safe(e: &Env, index_address: &Address) -> IndexCredits {
    get_index_credits(e, index_address).unwrap_or_else(|| {
        // Create default credits for new index
        IndexCredits::new(PortfolioTier::Micro, 50_000, e.ledger().timestamp())
    })
}

// Index Registry
pub fn get_index_registry(e: &Env) -> Vec<Address> {
    bump_instance(e);
    e.storage().instance().get(&DataKey::IndexRegistry).unwrap_or(Vec::new(e))
}

pub fn add_to_index_registry(e: &Env, index_address: &Address) {
    let mut registry = get_index_registry(e);
    
    // Check if already exists
    for addr in registry.iter() {
        if &addr == index_address {
            return; // Already in registry
        }
    }
    
    registry.push_back(index_address.clone());
    bump_instance(e);
    e.storage().instance().set(&DataKey::IndexRegistry, &registry);
}

pub fn remove_from_index_registry(e: &Env, index_address: &Address) {
    let registry = get_index_registry(e);
    let mut new_registry = Vec::new(e);
    
    for addr in registry.iter() {
        if &addr != index_address {
            new_registry.push_back(addr);
        }
    }
    
    bump_instance(e);
    e.storage().instance().set(&DataKey::IndexRegistry, &new_registry);
}

// Authorized Indexes
pub fn is_index_authorized(e: &Env, index_address: &Address) -> bool {
    let key = DataKey::AuthorizedIndexes;
    let authorized: Map<Address, bool> = e.storage().instance().get(&key).unwrap_or(Map::new(e));
    authorized.get(index_address.clone()).unwrap_or(false)
}

pub fn set_index_authorization(e: &Env, index_address: &Address, authorized: bool) {
    let key = DataKey::AuthorizedIndexes;
    let mut auth_map: Map<Address, bool> = e.storage().instance().get(&key).unwrap_or(Map::new(e));
    auth_map.set(index_address.clone(), authorized);
    
    bump_instance(e);
    e.storage().instance().set(&key, &auth_map);
    
    if authorized {
        add_to_index_registry(e, index_address);
    } else {
        remove_from_index_registry(e, index_address);
    }
}

// Asset Balance Tracking
pub fn get_total_asset_balance(e: &Env, asset: &Symbol) -> u128 {
    let key = DataKey::TotalAssetBalance(asset.clone());
    e.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_total_asset_balance(e: &Env, asset: &Symbol, balance: u128) {
    let key = DataKey::TotalAssetBalance(asset.clone());
    e.storage().persistent().set(&key, &balance);
    e.storage().persistent().extend_ttl(&key, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn increase_asset_balance(e: &Env, asset: &Symbol, amount: u128) {
    let current = get_total_asset_balance(e, asset);
    set_total_asset_balance(e, asset, current + amount);
}

pub fn decrease_asset_balance(e: &Env, asset: &Symbol, amount: u128) -> Result<(), MixerError> {
    let current = get_total_asset_balance(e, asset);
    if current < amount {
        return Err(MixerError::InsufficientMixerBalance);
    }
    set_total_asset_balance(e, asset, current - amount);
    Ok(())
}

// Supported Assets
pub fn get_supported_assets(e: &Env) -> Vec<Symbol> {
    bump_instance(e);
    e.storage().instance().get(&DataKey::SupportedAssets).unwrap_or(Vec::new(e))
}

pub fn set_supported_assets(e: &Env, assets: &Vec<Symbol>) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::SupportedAssets, assets);
}

pub fn is_asset_supported(e: &Env, asset: &Symbol) -> bool {
    let supported = get_supported_assets(e);
    for supported_asset in supported.iter() {
        if &supported_asset == asset {
            return true;
        }
    }
    false
}

// Withdrawal Request Management
pub fn get_withdrawal_counter(e: &Env) -> u32 {
    bump_instance(e);
    e.storage().instance().get(&DataKey::WithdrawalCounter).unwrap_or(0)
}

pub fn increment_withdrawal_counter(e: &Env) -> u32 {
    let counter = get_withdrawal_counter(e) + 1;
    bump_instance(e);
    e.storage().instance().set(&DataKey::WithdrawalCounter, &counter);
    counter
}

pub fn get_withdrawal_request(e: &Env, request_id: u32) -> Option<WithdrawalRequest> {
    let key = DataKey::WithdrawalRequest(request_id);
    e.storage().persistent().get(&key).map(|request| {
        bump_persistent(e, &key);
        request
    })
}

pub fn set_withdrawal_request(e: &Env, request: &WithdrawalRequest) {
    let key = DataKey::WithdrawalRequest(request.request_id);
    e.storage().persistent().set(&key, request);
    e.storage().persistent().extend_ttl(&key, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_pending_withdrawals(e: &Env) -> Vec<u32> {
    bump_instance(e);
    e.storage().instance().get(&DataKey::PendingWithdrawals).unwrap_or(Vec::new(e))
}

pub fn add_pending_withdrawal(e: &Env, request_id: u32) {
    let mut pending = get_pending_withdrawals(e);
    pending.push_back(request_id);
    bump_instance(e);
    e.storage().instance().set(&DataKey::PendingWithdrawals, &pending);
}

pub fn remove_pending_withdrawal(e: &Env, request_id: u32) {
    let pending = get_pending_withdrawals(e);
    let mut new_pending = Vec::new(e);
    
    for id in pending.iter() {
        if id != request_id {
            new_pending.push_back(id);
        }
    }
    
    bump_instance(e);
    e.storage().instance().set(&DataKey::PendingWithdrawals, &new_pending);
}

// Deposit Tracking
pub fn get_deposit_counter(e: &Env) -> u32 {
    bump_instance(e);
    e.storage().instance().get(&DataKey::DepositCounter).unwrap_or(0)
}

pub fn increment_deposit_counter(e: &Env) -> u32 {
    let counter = get_deposit_counter(e) + 1;
    bump_instance(e);
    e.storage().instance().set(&DataKey::DepositCounter, &counter);
    counter
}

pub fn set_deposit_record(e: &Env, record: &DepositRecord) {
    let key = DataKey::DepositRecord(record.deposit_id);
    e.storage().persistent().set(&key, record);
    e.storage().persistent().extend_ttl(&key, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

// Anonymity Set
pub fn get_anonymity_set_size(e: &Env) -> u32 {
    get_index_registry(e).len()
}

// Tier Distribution
pub fn get_tier_distribution(e: &Env) -> Vec<(PortfolioTier, u32)> {
    bump_instance(e);
    e.storage().instance().get(&DataKey::TierDistribution).unwrap_or(Vec::new(e))
}

pub fn update_tier_distribution(e: &Env) {
    let registry = get_index_registry(e);
    let mut distribution = Map::new(e);
    
    // Initialize all tiers to 0
    distribution.set(PortfolioTier::Micro, 0u32);
    distribution.set(PortfolioTier::Small, 0u32);
    distribution.set(PortfolioTier::Medium, 0u32);
    distribution.set(PortfolioTier::Large, 0u32);
    distribution.set(PortfolioTier::Whale, 0u32);
    distribution.set(PortfolioTier::Megalodon, 0u32);
    
    // Count indexes by tier
    for index_address in registry.iter() {
        if let Some(credits) = get_index_credits(e, &index_address) {
            let current_count = distribution.get(credits.current_tier.clone()).unwrap_or(0);
            distribution.set(credits.current_tier.clone(), current_count + 1);
        }
    }
    
    // Convert to Vec for storage
    let mut tier_vec = Vec::new(e);
    tier_vec.push_back((PortfolioTier::Micro, distribution.get(PortfolioTier::Micro).unwrap_or(0)));
    tier_vec.push_back((PortfolioTier::Small, distribution.get(PortfolioTier::Small).unwrap_or(0)));
    tier_vec.push_back((PortfolioTier::Medium, distribution.get(PortfolioTier::Medium).unwrap_or(0)));
    tier_vec.push_back((PortfolioTier::Large, distribution.get(PortfolioTier::Large).unwrap_or(0)));
    tier_vec.push_back((PortfolioTier::Whale, distribution.get(PortfolioTier::Whale).unwrap_or(0)));
    tier_vec.push_back((PortfolioTier::Megalodon, distribution.get(PortfolioTier::Megalodon).unwrap_or(0)));
    
    bump_instance(e);
    e.storage().instance().set(&DataKey::TierDistribution, &tier_vec);
}