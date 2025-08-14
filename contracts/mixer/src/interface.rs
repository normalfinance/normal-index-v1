use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};
use crate::types::{
    IndexCredits, WithdrawalRequest, DepositRecord, MixerConfig, MixerStats, PortfolioTier,
    AssetAllocation, WithdrawalStatus, BatchDepositRequest, BatchWithdrawalResult,
};

pub trait MixerTrait {
    /// Initialize the mixer contract
    fn initialize(e: Env, admin: Address, config: MixerConfig, supported_assets: Vec<Symbol>);

    /// Deposit assets from an index to the mixer
    fn deposit_from_index(
        e: Env,
        index_address: Address,
        asset_amounts: Vec<(Symbol, u128)>,
        total_usd_value: u128,
    ) -> u32; // Returns deposit_id

    /// Request USD-denominated withdrawal
    fn request_withdrawal(
        e: Env,
        index_address: Address,
        usd_amount: u128,
        preferred_assets: Vec<Symbol>,
    ) -> u32; // Returns request_id

    /// Execute a pending withdrawal
    fn execute_withdrawal(e: Env, request_id: u32) -> Vec<AssetAllocation>;

    /// Cancel a pending withdrawal
    fn cancel_withdrawal(e: Env, request_id: u32, index_address: Address);

    /// Get index credits (tier and allowances)
    fn get_index_credits(e: Env, index_address: Address) -> IndexCredits;

    /// Get withdrawal request details
    fn get_withdrawal_request(e: Env, request_id: u32) -> WithdrawalRequest;

    /// Get mixer statistics
    fn get_mixer_stats(e: Env) -> MixerStats;

    /// Get total balance for a specific asset
    fn get_asset_balance(e: Env, asset: Symbol) -> u128;

    /// Get all supported assets
    fn get_supported_assets(e: Env) -> Vec<Symbol>;

    /// Batch deposit from multiple indexes for enhanced privacy
    fn batch_deposit_from_indexes(e: Env, deposits: Vec<BatchDepositRequest>) -> Vec<u32>;

    /// Batch execute withdrawals for enhanced privacy
    fn batch_execute_withdrawals(e: Env, request_ids: Vec<u32>) -> Vec<BatchWithdrawalResult>;
}

pub trait MixerAdminTrait {
    /// Update mixer configuration
    fn update_config(e: Env, admin: Address, config: MixerConfig);

    /// Authorize an index to use the mixer
    fn authorize_index(e: Env, admin: Address, index_address: Address);

    /// Deauthorize an index from using the mixer
    fn deauthorize_index(e: Env, admin: Address, index_address: Address);

    /// Add supported asset
    fn add_supported_asset(e: Env, admin: Address, asset: Symbol);

    /// Remove supported asset
    fn remove_supported_asset(e: Env, admin: Address, asset: Symbol);

    /// Update asset price for USD calculations
    fn update_asset_price(e: Env, admin: Address, asset: Symbol, price_usd: u128);

    /// Emergency pause mixer
    fn pause_mixer(e: Env, admin: Address);

    /// Resume mixer operations
    fn resume_mixer(e: Env, admin: Address);

    /// Transfer admin rights
    fn transfer_admin(e: Env, current_admin: Address, new_admin: Address);
}

pub trait MixerQueryTrait {
    /// Check if index is authorized
    fn is_index_authorized(e: Env, index_address: Address) -> bool;

    /// Get pending withdrawals for an index
    fn get_pending_withdrawals(e: Env, index_address: Address) -> Vec<u32>;

    /// Get deposit history for an index
    fn get_deposit_history(e: Env, index_address: Address) -> Vec<DepositRecord>;

    /// Calculate tier for USD amount
    fn calculate_tier_for_amount(e: Env, usd_amount: u128) -> PortfolioTier;

    /// Get effective tier (considering grace period)
    fn get_effective_tier(e: Env, index_address: Address) -> PortfolioTier;

    /// Check withdrawal eligibility
    fn can_withdraw(e: Env, index_address: Address, usd_amount: u128) -> bool;

    /// Get anonymity set size
    fn get_anonymity_set_size(e: Env) -> u32;

    /// Get withdrawal rate limit status
    fn get_withdrawal_rate_limit_status(e: Env) -> (u32, u64); // (current_count, reset_time)
}

pub trait MixerPrivacyTrait {
    /// Get tier distribution (public stats)
    fn get_tier_distribution(e: Env) -> Vec<(PortfolioTier, u32)>;

    /// Get aggregated mixer totals
    fn get_mixer_totals(e: Env) -> Vec<(Symbol, u128)>;

    /// Calculate portfolio tier with privacy noise
    fn calculate_tier_with_privacy(
        e: Env,
        usd_amount: u128,
        index_address: Address,
    ) -> PortfolioTier;

    /// Get public mixer metrics (no individual data)
    fn get_public_metrics(e: Env) -> MixerStats;

    /// Apply tier momentum for privacy enhancements
    fn apply_tier_momentum(e: Env, index_address: Address) -> PortfolioTier;
}