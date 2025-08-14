use soroban_sdk::{Address, Env, Symbol, Vec, panic_with_error};
use crate::errors::MixerError;
use crate::events::{MixerContract, MixerEvents};
use crate::storage::*;
use crate::types::*;

/// Batch deposit from multiple indexes for enhanced privacy
pub fn batch_deposit_from_indexes(
    e: &Env,
    deposits: Vec<BatchDepositRequest>,
) -> Vec<u32> {
    let config = get_mixer_config(e);
    if !config.is_active {
        panic_with_error!(e, MixerError::NotActive);
    }
    
    let mut deposit_ids = Vec::new(e);
    let events = MixerContract;
    
    // Process all deposits together for better privacy
    for deposit_request in deposits.iter() {
        // Verify authorization
        if !is_index_authorized(e, &deposit_request.index_address) {
            panic_with_error!(e, MixerError::UnauthorizedIndex);
        }
        
        // Update asset balances
        for (asset, amount) in deposit_request.asset_amounts.iter() {
            increase_asset_balance(e, &asset, amount);
        }
        
        // Update index credits
        let mut credits = get_index_credits_safe(e, &deposit_request.index_address);
        credits.cumulative_deposits_usd += deposit_request.total_usd_value;
        credits.deposit_count += 1;
        
        // Calculate new tier with privacy noise
        let new_tier = calculate_tier_with_privacy_noise(
            e,
            credits.cumulative_deposits_usd,
            &deposit_request.index_address,
            config.tier_noise_factor,
        );
        
        let old_tier = credits.current_tier.clone();
        let tier_upgraded = new_tier.level() > credits.current_tier.level();
        
        if tier_upgraded {
            credits.current_tier = new_tier.clone();
            credits.peak_tier = new_tier.clone();
            credits.tier_allowance = new_tier.max_usd_value();
            credits.peak_tier_timestamp = e.ledger().timestamp();
            credits.tier_decay_timestamp = e.ledger().timestamp();
        }
        
        set_index_credits(e, &deposit_request.index_address, &credits);
        
        // Create deposit record
        let deposit_id = increment_deposit_counter(e);
        let deposit_record = DepositRecord {
            index_address: deposit_request.index_address.clone(),
            total_usd_value: deposit_request.total_usd_value,
            timestamp: e.ledger().timestamp(),
            deposit_id,
            tier_achieved: if tier_upgraded { Some(new_tier.clone()) } else { None },
        };
        set_deposit_record(e, &deposit_record);
        
        deposit_ids.push_back(deposit_id);
        
        // Emit events
        events.deposit_to_mixer(
            deposit_request.index_address.clone(),
            deposit_request.total_usd_value,
            if tier_upgraded { Some(new_tier.clone()) } else { None },
            deposit_id,
        );
        
        if tier_upgraded {
            events.tier_upgraded(
                deposit_request.index_address.clone(),
                old_tier,
                new_tier,
                credits.tier_allowance,
            );
        }
    }
    
    // Update tier distribution once for all deposits
    update_tier_distribution(e);
    
    deposit_ids
}

/// Batch withdrawal execution for enhanced privacy
pub fn batch_execute_withdrawals(
    e: &Env,
    request_ids: Vec<u32>,
) -> Vec<BatchWithdrawalResult> {
    let mut results = Vec::new(e);
    let events = MixerContract;
    
    // Check all requests first
    for request_id in request_ids.iter() {
        let request = get_withdrawal_request(e, request_id);
        if let Some(req) = request {
            if req.status != WithdrawalStatus::Pending {
                panic_with_error!(e, MixerError::WithdrawalNotPending);
            }
            if e.ledger().timestamp() < req.earliest_execution {
                panic_with_error!(e, MixerError::WithdrawalTooEarly);
            }
        } else {
            panic_with_error!(e, MixerError::WithdrawalRequestNotFound);
        }
    }
    
    // Execute all withdrawals together
    for request_id in request_ids.iter() {
        let mut request = get_withdrawal_request(e, request_id).unwrap();
        
        // Update status
        request.status = WithdrawalStatus::Processing;
        set_withdrawal_request(e, &request);
        
        // Calculate allocation
        let asset_allocations = calculate_withdrawal_allocation(
            e,
            request.usd_amount,
            &request.preferred_assets,
        );
        
        match asset_allocations {
            Ok(allocations) => {
                // Update balances
                for allocation in allocations.iter() {
                    if let Err(_) = decrease_asset_balance(e, &allocation.asset, allocation.amount) {
                        // Rollback and mark as failed
                        request.status = WithdrawalStatus::Pending;
                        set_withdrawal_request(e, &request);
                        
                        results.push_back(BatchWithdrawalResult {
                            request_id,
                            success: false,
                            allocations: Vec::new(e),
                            error_code: Some(MixerError::InsufficientMixerBalance as u32),
                        });
                        continue;
                    }
                }
                
                // Update credits
                let mut credits = get_index_credits(e, &request.index_address).unwrap();
                credits.used_allowance += request.usd_amount;
                credits.withdrawal_count += 1;
                set_index_credits(e, &request.index_address, &credits);
                
                // Mark as completed
                request.status = WithdrawalStatus::Completed;
                set_withdrawal_request(e, &request);
                remove_pending_withdrawal(e, request_id);
                
                // Execute transfers
                let mut transfer_assets = Vec::new(e);
                for allocation in allocations.iter() {
                    transfer_assets.push_back((allocation.asset.clone(), allocation.amount));
                }
                
                execute_token_transfers(e, &request.index_address, &transfer_assets);
                
                // Emit event
                events.withdrawal_executed(
                    request.index_address,
                    request.usd_amount,
                    transfer_assets,
                    request_id,
                );
                
                results.push_back(BatchWithdrawalResult {
                    request_id,
                    success: true,
                    allocations,
                    error_code: None,
                });
            },
            Err(error) => {
                request.status = WithdrawalStatus::Pending;
                set_withdrawal_request(e, &request);
                
                results.push_back(BatchWithdrawalResult {
                    request_id,
                    success: false,
                    allocations: Vec::new(e),
                    error_code: Some(error as u32),
                });
            }
        }
    }
    
    results
}

/// Enhanced privacy features
pub fn apply_tier_momentum(e: &Env, index_address: &Address) -> PortfolioTier {
    let credits = get_index_credits_safe(e, index_address);
    let config = get_mixer_config(e);
    let current_time = e.ledger().timestamp();
    
    // Calculate momentum factor based on activity
    let activity_factor = calculate_activity_momentum(&credits);
    let time_factor = calculate_time_momentum(&credits, current_time, config.tier_grace_period);
    
    // Apply momentum to tier calculation
    let base_tier = credits.current_tier.clone();
    let peak_tier = credits.peak_tier.clone();
    
    if time_factor > 0.7 && activity_factor > 0.5 {
        // High momentum - maintain peak tier
        peak_tier
    } else if time_factor > 0.4 {
        // Medium momentum - interpolate between current and peak
        if peak_tier.level() > base_tier.level() {
            // Return tier one level above current (but not exceeding peak)
            get_tier_by_level((base_tier.level() + 1).min(peak_tier.level()))
        } else {
            base_tier
        }
    } else {
        // Low momentum - use current tier
        base_tier
    }
}

fn calculate_activity_momentum(credits: &IndexCredits) -> f64 {
    // Calculate based on deposit/withdrawal frequency
    let total_activity = credits.deposit_count + credits.withdrawal_count;
    if total_activity == 0 {
        return 0.0;
    }
    
    // Higher activity = higher momentum (capped at 1.0)
    (total_activity as f64 / 10.0).min(1.0)
}

fn calculate_time_momentum(credits: &IndexCredits, current_time: u64, grace_period: u64) -> f64 {
    let time_since_peak = current_time.saturating_sub(credits.peak_tier_timestamp);
    if time_since_peak >= grace_period {
        return 0.0;
    }
    
    // Linear decay from 1.0 to 0.0 over grace period
    1.0 - (time_since_peak as f64 / grace_period as f64)
}

fn get_tier_by_level(level: u32) -> PortfolioTier {
    match level {
        1 => PortfolioTier::Micro,
        2 => PortfolioTier::Small,
        3 => PortfolioTier::Medium,
        4 => PortfolioTier::Large,
        5 => PortfolioTier::Whale,
        6 => PortfolioTier::Megalodon,
        _ => PortfolioTier::Micro,
    }
}

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
        usd_amount.saturating_add((usd_amount * noise_percent as u128) / 10000)
    } else {
        usd_amount.saturating_sub((usd_amount * (-noise_percent) as u128) / 10000)
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

fn calculate_withdrawal_allocation(
    e: &Env,
    usd_amount: u128,
    preferred_assets: &Vec<Symbol>,
) -> Result<Vec<AssetAllocation>, MixerError> {
    let mut allocations = Vec::new(e);
    
    if preferred_assets.is_empty() {
        return Err(MixerError::InvalidPreferredAssets);
    }
    
    // Distribute among preferred assets based on availability and preferences
    let per_asset_usd = usd_amount / preferred_assets.len() as u128;
    
    for asset in preferred_assets.iter() {
        let asset_price = get_mock_asset_price(&asset);
        let asset_amount = per_asset_usd * 10000000 / asset_price; // Adjust for decimals
        
        // Check availability
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
    // Mock prices - would integrate with real oracle
    // For now, return simple price based on asset
    // In a real implementation, this would query an oracle
    1_0000000 // Default to $1 with 7 decimals
}

fn execute_token_transfers(e: &Env, to_address: &Address, assets: &Vec<(Symbol, u128)>) {
    // Placeholder for actual token transfers
    // Would integrate with Stellar token contracts
    for (_asset, _amount) in assets.iter() {
        // Token transfer logic would go here
    }
}