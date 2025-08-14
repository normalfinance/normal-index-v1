use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec, panic_with_error};

use crate::batch::*;
use crate::errors::MixerError;
use crate::events::{MixerContract, MixerEvents};
use crate::interface::{MixerTrait, MixerAdminTrait, MixerQueryTrait, MixerPrivacyTrait};
use crate::storage::*;
use crate::types::*;

#[contract]
pub struct Mixer;

#[contractimpl]
impl MixerTrait for Mixer {
    fn initialize(e: Env, admin: Address, config: MixerConfig, supported_assets: Vec<Symbol>) {
        admin.require_auth();
        
        if is_initialized(&e) {
            panic_with_error!(&e, MixerError::NotInitialized);
        }
        
        set_admin(&e, &admin);
        set_mixer_config(&e, &config);
        set_supported_assets(&e, &supported_assets);
        set_initialized(&e);
        
        let events = MixerContract;
        events.mixer_config_updated(admin);
    }

    fn deposit_from_index(
        e: Env,
        index_address: Address,
        asset_amounts: Vec<(Symbol, u128)>,
        total_usd_value: u128,
    ) -> u32 {
        // Verify mixer is active
        let config = get_mixer_config(&e);
        if !config.is_active {
            panic_with_error!(&e, MixerError::NotActive);
        }
        
        // Verify index is authorized
        if !is_index_authorized(&e, &index_address) {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        // Validate deposit amount
        if total_usd_value == 0 {
            panic_with_error!(&e, MixerError::InvalidDepositAmount);
        }
        
        // Validate assets are supported
        for (asset, amount) in asset_amounts.iter() {
            if amount == 0 {
                panic_with_error!(&e, MixerError::InvalidDepositAmount);
            }
            if !is_asset_supported(&e, &asset) {
                panic_with_error!(&e, MixerError::UnsupportedAsset);
            }
        }
        
        // Update asset balances in mixer
        for (asset, amount) in asset_amounts.iter() {
            increase_asset_balance(&e, &asset, amount);
        }
        
        // Get or create index credits
        let mut credits = get_index_credits_safe(&e, &index_address);
        
        // Update cumulative deposits
        credits.cumulative_deposits_usd += total_usd_value;
        credits.deposit_count += 1;
        
        // Calculate new tier with privacy noise
        let new_tier = calculate_tier_with_privacy_noise(
            &e,
            credits.cumulative_deposits_usd,
            &index_address,
            config.tier_noise_factor,
        );
        
        let old_tier = credits.current_tier.clone();
        let tier_upgraded = new_tier.level() > credits.current_tier.level();
        
        // Update tier if upgraded
        if tier_upgraded {
            credits.current_tier = new_tier.clone();
            credits.peak_tier = new_tier.clone();
            credits.tier_allowance = new_tier.max_usd_value();
            credits.peak_tier_timestamp = e.ledger().timestamp();
            credits.tier_decay_timestamp = e.ledger().timestamp();
        }
        
        // Save updated credits
        set_index_credits(&e, &index_address, &credits);
        
        // Create deposit record
        let deposit_id = increment_deposit_counter(&e);
        let deposit_record = DepositRecord {
            index_address: index_address.clone(),
            total_usd_value,
            timestamp: e.ledger().timestamp(),
            deposit_id,
            tier_achieved: if tier_upgraded { Some(new_tier.clone()) } else { None },
        };
        set_deposit_record(&e, &deposit_record);
        
        // Update tier distribution
        update_tier_distribution(&e);
        
        // Emit events
        let events = MixerContract;
        events.deposit_to_mixer(
            index_address.clone(),
            total_usd_value,
            if tier_upgraded { Some(new_tier.clone()) } else { None },
            deposit_id,
        );
        
        if tier_upgraded {
            events.tier_upgraded(
                index_address,
                old_tier,
                new_tier,
                credits.tier_allowance,
            );
        }
        
        deposit_id
    }

    fn request_withdrawal(
        e: Env,
        index_address: Address,
        usd_amount: u128,
        preferred_assets: Vec<Symbol>,
    ) -> u32 {
        // Verify mixer is active
        let config = get_mixer_config(&e);
        if !config.is_active {
            panic_with_error!(&e, MixerError::NotActive);
        }
        
        // Verify index is authorized
        if !is_index_authorized(&e, &index_address) {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        // Validate withdrawal amount
        if usd_amount == 0 {
            panic_with_error!(&e, MixerError::InvalidWithdrawalAmount);
        }
        
        // Check anonymity set
        if get_anonymity_set_size(&e) < config.min_anonymity_set {
            panic_with_error!(&e, MixerError::InsufficientAnonymitySet);
        }
        
        // Get index credits
        let credits = get_index_credits(&e, &index_address)
            .ok_or(MixerError::IndexNotFound)
            .unwrap();
        
        // Get effective tier (considering grace period)
        let effective_tier = get_effective_tier_internal(&e, &credits);
        let effective_allowance = effective_tier.max_usd_value();
        
        // Check if withdrawal is within allowance
        if credits.used_allowance + usd_amount > effective_allowance {
            panic_with_error!(&e, MixerError::InsufficientTierAllowance);
        }
        
        // Validate preferred assets
        for asset in preferred_assets.iter() {
            if !is_asset_supported(&e, &asset) {
                panic_with_error!(&e, MixerError::UnsupportedAsset);
            }
        }
        
        // Check for existing pending withdrawal
        let pending = get_pending_withdrawals(&e);
        for request_id in pending.iter() {
            if let Some(request) = get_withdrawal_request(&e, request_id) {
                if request.index_address == index_address && 
                   request.status == WithdrawalStatus::Pending {
                    panic_with_error!(&e, MixerError::PendingWithdrawalExists);
                }
            }
        }
        
        // Create withdrawal request
        let request_id = increment_withdrawal_counter(&e);
        let current_time = e.ledger().timestamp();
        let withdrawal_request = WithdrawalRequest {
            index_address: index_address.clone(),
            usd_amount,
            preferred_assets: preferred_assets.clone(),
            status: WithdrawalStatus::Pending,
            request_timestamp: current_time,
            earliest_execution: current_time + config.withdrawal_delay,
            request_id,
        };
        
        set_withdrawal_request(&e, &withdrawal_request);
        add_pending_withdrawal(&e, request_id);
        
        // Emit event
        let events = MixerContract;
        events.withdrawal_requested(
            index_address,
            usd_amount,
            preferred_assets,
            request_id,
        );
        
        request_id
    }

    fn execute_withdrawal(e: Env, request_id: u32) -> Vec<AssetAllocation> {
        let mut request = get_withdrawal_request(&e, request_id)
            .ok_or(MixerError::WithdrawalRequestNotFound)
            .unwrap();
        
        // Verify request is pending
        if request.status != WithdrawalStatus::Pending {
            panic_with_error!(&e, MixerError::WithdrawalNotPending);
        }
        
        // Check if delay period has passed
        if e.ledger().timestamp() < request.earliest_execution {
            panic_with_error!(&e, MixerError::WithdrawalTooEarly);
        }
        
        // Check rate limiting
        check_withdrawal_rate_limit(&e).unwrap();
        
        // Update request status
        request.status = WithdrawalStatus::Processing;
        set_withdrawal_request(&e, &request);
        
        // Calculate asset allocation based on preferences and availability
        let asset_allocations = calculate_withdrawal_allocation(
            &e,
            request.usd_amount,
            &request.preferred_assets,
        ).unwrap();
        
        // Update mixer balances
        for allocation in asset_allocations.iter() {
            decrease_asset_balance(&e, &allocation.asset, allocation.amount).unwrap();
        }
        
        // Update index credits
        let mut credits = get_index_credits(&e, &request.index_address).unwrap();
        credits.used_allowance += request.usd_amount;
        credits.withdrawal_count += 1;
        set_index_credits(&e, &request.index_address, &credits);
        
        // Mark request as completed
        request.status = WithdrawalStatus::Completed;
        set_withdrawal_request(&e, &request);
        remove_pending_withdrawal(&e, request_id);
        
        // Convert allocations to transfer format
        let mut transfer_assets = Vec::new(&e);
        for allocation in asset_allocations.iter() {
            transfer_assets.push_back((allocation.asset.clone(), allocation.amount));
        }
        
        // Execute actual token transfers (placeholder - would integrate with token contracts)
        execute_token_transfers(&e, &request.index_address, &transfer_assets);
        
        // Emit event
        let events = MixerContract;
        events.withdrawal_executed(
            request.index_address,
            request.usd_amount,
            transfer_assets,
            request_id,
        );
        
        asset_allocations
    }

    fn cancel_withdrawal(e: Env, request_id: u32, index_address: Address) {
        index_address.require_auth();
        
        let mut request = get_withdrawal_request(&e, request_id)
            .ok_or(MixerError::WithdrawalRequestNotFound)
            .unwrap();
        
        // Verify request belongs to this index
        if request.index_address != index_address {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        // Verify request is still pending
        if request.status != WithdrawalStatus::Pending {
            panic_with_error!(&e, MixerError::WithdrawalNotPending);
        }
        
        // Update status
        request.status = WithdrawalStatus::Cancelled;
        set_withdrawal_request(&e, &request);
        remove_pending_withdrawal(&e, request_id);
        
        // Emit event
        let events = MixerContract;
        events.withdrawal_status_changed(
            request_id,
            index_address,
            WithdrawalStatus::Pending,
            WithdrawalStatus::Cancelled,
        );
    }

    fn get_index_credits(e: Env, index_address: Address) -> IndexCredits {
        get_index_credits_safe(&e, &index_address)
    }

    fn get_withdrawal_request(e: Env, request_id: u32) -> WithdrawalRequest {
        get_withdrawal_request(&e, request_id)
            .ok_or(MixerError::WithdrawalRequestNotFound)
            .unwrap()
    }

    fn get_mixer_stats(e: Env) -> MixerStats {
        let registry = get_index_registry(&e);
        let tier_distribution = get_tier_distribution(&e);
        
        // Calculate total USD value
        let mut total_usd_value = 0u128;
        for index_address in registry.iter() {
            if let Some(credits) = get_index_credits(&e, &index_address) {
                total_usd_value += credits.cumulative_deposits_usd - credits.used_allowance;
            }
        }
        
        MixerStats {
            total_indexes: registry.len(),
            total_usd_value,
            total_deposits: get_deposit_counter(&e),
            total_withdrawals: get_withdrawal_counter(&e),
            tier_distribution,
            avg_withdrawal_delay: get_mixer_config(&e).withdrawal_delay,
        }
    }

    fn get_asset_balance(e: Env, asset: Symbol) -> u128 {
        get_total_asset_balance(&e, &asset)
    }

    fn get_supported_assets(e: Env) -> Vec<Symbol> {
        get_supported_assets(&e)
    }

    fn batch_deposit_from_indexes(e: Env, deposits: Vec<BatchDepositRequest>) -> Vec<u32> {
        batch_deposit_from_indexes(&e, deposits)
    }

    fn batch_execute_withdrawals(e: Env, request_ids: Vec<u32>) -> Vec<BatchWithdrawalResult> {
        batch_execute_withdrawals(&e, request_ids)
    }
}

// Helper functions
fn calculate_tier_with_privacy_noise(
    e: &Env,
    usd_amount: u128,
    index_address: &Address,
    noise_factor: u32,
) -> PortfolioTier {
    // Add deterministic noise based on address
    // Use a simple deterministic approach instead of hashing  
    let noise_seed = (usd_amount % 10000) as u32;
    
    // Apply ±noise_factor% variation
    let noise_percent = (noise_seed % (noise_factor * 2)) as i32 - noise_factor as i32;
    let adjusted_amount = if noise_percent > 0 {
        usd_amount + (usd_amount * noise_percent as u128) / 10000
    } else {
        usd_amount - (usd_amount * (-noise_percent) as u128) / 10000
    };
    
    calculate_tier_for_amount(adjusted_amount)
}

fn calculate_tier_for_amount(usd_amount: u128) -> PortfolioTier {
    if usd_amount >= PortfolioTier::Megalodon.min_usd_value() {
        PortfolioTier::Megalodon
    } else if usd_amount >= PortfolioTier::Whale.min_usd_value() {
        PortfolioTier::Whale
    } else if usd_amount >= PortfolioTier::Large.min_usd_value() {
        PortfolioTier::Large
    } else if usd_amount >= PortfolioTier::Medium.min_usd_value() {
        PortfolioTier::Medium
    } else if usd_amount >= PortfolioTier::Small.min_usd_value() {
        PortfolioTier::Small
    } else {
        PortfolioTier::Micro
    }
}

fn get_effective_tier_internal(e: &Env, credits: &IndexCredits) -> PortfolioTier {
    let config = get_mixer_config(e);
    let current_time = e.ledger().timestamp();
    let time_since_peak = current_time - credits.peak_tier_timestamp;
    
    if time_since_peak < config.tier_grace_period {
        // Still in grace period, use peak tier
        credits.peak_tier.clone()
    } else {
        // Grace period expired, use current tier
        credits.current_tier.clone()
    }
}

fn check_withdrawal_rate_limit(e: &Env) -> Result<(), MixerError> {
    // Placeholder for rate limiting logic
    // Would track withdrawal counts per time period
    Ok(())
}

fn calculate_withdrawal_allocation(
    e: &Env,
    usd_amount: u128,
    preferred_assets: &Vec<Symbol>,
) -> Result<Vec<AssetAllocation>, MixerError> {
    // Simplified allocation - would integrate with price oracles
    let mut allocations = Vec::new(e);
    
    // For now, distribute evenly among preferred assets
    let per_asset_usd = usd_amount / preferred_assets.len() as u128;
    
    for asset in preferred_assets.iter() {
        // Mock price calculation (would use real oracle)
        let asset_price = get_mock_asset_price(&asset);
        let asset_amount = per_asset_usd / asset_price;
        
        // Check if mixer has enough of this asset
        let available = get_total_asset_balance(e, &asset);
        if available < asset_amount {
            return Err(MixerError::InsufficientMixerBalance);
        }
        
        allocations.push_back(AssetAllocation {
            asset: asset.clone(),
            amount: asset_amount,
            usd_value: per_asset_usd,
        });
    }
    
    Ok(allocations)
}

fn get_mock_asset_price(asset: &Symbol) -> u128 {
    // Mock prices for testing - would integrate with real price oracle
    // In a real implementation, this would query an oracle
    1_0000000 // Default to $1 with 7 decimals
}

fn execute_token_transfers(e: &Env, to_address: &Address, assets: &Vec<(Symbol, u128)>) {
    // Placeholder for actual token transfers
    // Would integrate with Stellar token contracts
    for (asset, amount) in assets.iter() {
        // Token transfer logic would go here
        // For now, just emit log
    }
}

#[contractimpl]
impl MixerAdminTrait for Mixer {
    fn update_config(e: Env, admin: Address, config: MixerConfig) {
        admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        set_mixer_config(&e, &config);
        
        let events = MixerContract;
        events.mixer_config_updated(admin);
    }
    
    fn authorize_index(e: Env, admin: Address, index_address: Address) {
        admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        set_index_authorization(&e, &index_address, true);
        
        let events = MixerContract;
        events.index_authorized(index_address, admin);
    }
    
    fn deauthorize_index(e: Env, admin: Address, index_address: Address) {
        admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        set_index_authorization(&e, &index_address, false);
        
        let events = MixerContract;
        events.index_deauthorized(index_address, admin);
    }
    
    fn add_supported_asset(e: Env, admin: Address, asset: Symbol) {
        admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        let mut assets = get_supported_assets(&e);
        
        // Check if already supported
        for supported_asset in assets.iter() {
            if supported_asset == asset {
                return; // Already supported
            }
        }
        
        assets.push_back(asset);
        set_supported_assets(&e, &assets);
    }
    
    fn remove_supported_asset(e: Env, admin: Address, asset: Symbol) {
        admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        let assets = get_supported_assets(&e);
        let mut new_assets = Vec::new(&e);
        
        for supported_asset in assets.iter() {
            if supported_asset != asset {
                new_assets.push_back(supported_asset);
            }
        }
        
        set_supported_assets(&e, &new_assets);
    }
    
    fn update_asset_price(e: Env, admin: Address, asset: Symbol, price_usd: u128) {
        admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        // Would store price in oracle integration
        // For now, placeholder
    }
    
    fn pause_mixer(e: Env, admin: Address) {
        admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        let mut config = get_mixer_config(&e);
        config.is_active = false;
        set_mixer_config(&e, &config);
        
        let events = MixerContract;
        events.mixer_config_updated(admin);
    }
    
    fn resume_mixer(e: Env, admin: Address) {
        admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        let mut config = get_mixer_config(&e);
        config.is_active = true;
        set_mixer_config(&e, &config);
        
        let events = MixerContract;
        events.mixer_config_updated(admin);
    }
    
    fn transfer_admin(e: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        
        let stored_admin = get_admin(&e);
        if current_admin != stored_admin {
            panic_with_error!(&e, MixerError::UnauthorizedIndex);
        }
        
        set_admin(&e, &new_admin);
        
        let events = MixerContract;
        events.mixer_config_updated(new_admin);
    }
}

#[contractimpl]
impl MixerQueryTrait for Mixer {
    fn is_index_authorized(e: Env, index_address: Address) -> bool {
        is_index_authorized(&e, &index_address)
    }
    
    fn get_pending_withdrawals(e: Env, index_address: Address) -> Vec<u32> {
        let pending = get_pending_withdrawals(&e);
        let mut result = Vec::new(&e);
        
        for request_id in pending.iter() {
            if let Some(request) = get_withdrawal_request(&e, request_id) {
                if request.index_address == index_address {
                    result.push_back(request_id);
                }
            }
        }
        
        result
    }
    
    fn get_deposit_history(e: Env, index_address: Address) -> Vec<DepositRecord> {
        // Simplified - would need better indexing in production
        let mut history = Vec::new(&e);
        let total_deposits = get_deposit_counter(&e);
        
        for i in 1..=total_deposits {
            let key = crate::storage::DataKey::DepositRecord(i);
            if let Some(record) = e.storage().persistent().get::<crate::storage::DataKey, DepositRecord>(&key) {
                if record.index_address == index_address {
                    history.push_back(record);
                }
            }
        }
        
        history
    }
    
    fn calculate_tier_for_amount(e: Env, usd_amount: u128) -> PortfolioTier {
        calculate_tier_for_amount(usd_amount)
    }
    
    fn get_effective_tier(e: Env, index_address: Address) -> PortfolioTier {
        if let Some(credits) = get_index_credits(&e, &index_address) {
            get_effective_tier_internal(&e, &credits)
        } else {
            PortfolioTier::Micro
        }
    }
    
    fn can_withdraw(e: Env, index_address: Address, usd_amount: u128) -> bool {
        if let Some(credits) = get_index_credits(&e, &index_address) {
            let effective_tier = get_effective_tier_internal(&e, &credits);
            let effective_allowance = effective_tier.max_usd_value();
            credits.used_allowance + usd_amount <= effective_allowance
        } else {
            false
        }
    }
    
    fn get_anonymity_set_size(e: Env) -> u32 {
        get_anonymity_set_size(&e)
    }
    
    fn get_withdrawal_rate_limit_status(e: Env) -> (u32, u64) {
        // Placeholder - would implement proper rate limiting
        (0, 0)
    }
}

#[contractimpl]
impl MixerPrivacyTrait for Mixer {
    fn get_tier_distribution(e: Env) -> Vec<(PortfolioTier, u32)> {
        get_tier_distribution(&e)
    }
    
    fn get_mixer_totals(e: Env) -> Vec<(Symbol, u128)> {
        let assets = get_supported_assets(&e);
        let mut totals = Vec::new(&e);
        
        for asset in assets.iter() {
            let balance = get_total_asset_balance(&e, &asset);
            totals.push_back((asset, balance));
        }
        
        totals
    }
    
    fn calculate_tier_with_privacy(
        e: Env,
        usd_amount: u128,
        index_address: Address,
    ) -> PortfolioTier {
        let config = get_mixer_config(&e);
        calculate_tier_with_privacy_noise(&e, usd_amount, &index_address, config.tier_noise_factor)
    }
    
    fn get_public_metrics(e: Env) -> MixerStats {
        let registry = get_index_registry(&e);
        let tier_distribution = get_tier_distribution(&e);
        
        // Calculate aggregated totals only
        let assets = get_supported_assets(&e);
        let mut total_usd_value = 0u128;
        
        for asset in assets.iter() {
            let balance = get_total_asset_balance(&e, &asset);
            let price = get_mock_asset_price(&asset);
            total_usd_value += balance * price / 10000000; // Adjust for decimals
        }
        
        MixerStats {
            total_indexes: registry.len(),
            total_usd_value,
            total_deposits: get_deposit_counter(&e),
            total_withdrawals: get_withdrawal_counter(&e),
            tier_distribution,
            avg_withdrawal_delay: get_mixer_config(&e).withdrawal_delay,
        }
    }

    fn apply_tier_momentum(e: Env, index_address: Address) -> PortfolioTier {
        apply_tier_momentum(&e, &index_address)
    }
}