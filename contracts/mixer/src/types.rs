use soroban_sdk::{contracttype, Address, Symbol, Vec};

/// Portfolio tier based on USD value ranges
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PortfolioTier {
    /// $10K - $50K
    Micro,
    /// $50K - $250K  
    Small,
    /// $250K - $1M
    Medium,
    /// $1M - $5M
    Large,
    /// $5M - $25M
    Whale,
    /// $25M+
    Megalodon,
}

/// Range-based credits for privacy-preserving withdrawals
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexCredits {
    /// Current portfolio tier
    pub current_tier: PortfolioTier,
    /// Highest tier ever achieved
    pub peak_tier: PortfolioTier,
    /// Maximum USD withdrawable for current tier
    pub tier_allowance: u128,
    /// How much USD already withdrawn
    pub used_allowance: u128,
    /// When the peak tier was achieved
    pub peak_tier_timestamp: u64,
    /// When tier started decaying
    pub tier_decay_timestamp: u64,
    /// Number of deposits made
    pub deposit_count: u32,
    /// Number of withdrawals made
    pub withdrawal_count: u32,
    /// Total USD value ever deposited (for tier calculation)
    pub cumulative_deposits_usd: u128,
    /// Whether this index is active in the mixer
    pub is_active: bool,
}

/// Withdrawal request for USD-denominated withdrawals
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalRequest {
    /// Index requesting withdrawal
    pub index_address: Address,
    /// USD amount to withdraw
    pub usd_amount: u128,
    /// Preferred assets to receive
    pub preferred_assets: Vec<Symbol>,
    /// Current status of the request
    pub status: WithdrawalStatus,
    /// When request was created
    pub request_timestamp: u64,
    /// Earliest time withdrawal can be executed
    pub earliest_execution: u64,
    /// Unique request ID
    pub request_id: u32,
}

/// Status of withdrawal requests
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WithdrawalStatus {
    /// Request is pending execution
    Pending,
    /// Request is being processed
    Processing,
    /// Request has been completed
    Completed,
    /// Request has been cancelled
    Cancelled,
}

/// Deposit record for tracking mixer activity
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositRecord {
    /// Index that made the deposit
    pub index_address: Address,
    /// Total USD value of deposit
    pub total_usd_value: u128,
    /// When deposit was made
    pub timestamp: u64,
    /// Unique deposit ID
    pub deposit_id: u32,
    /// New tier achieved (if any)
    pub tier_achieved: Option<PortfolioTier>,
}

/// Asset allocation for withdrawals
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAllocation {
    /// Asset symbol
    pub asset: Symbol,
    /// Amount of this asset
    pub amount: u128,
    /// USD value of this allocation
    pub usd_value: u128,
}

/// Mixer configuration parameters
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MixerConfig {
    /// Minimum number of deposits before withdrawals allowed
    pub min_anonymity_set: u32,
    /// Base withdrawal delay in seconds
    pub withdrawal_delay: u64,
    /// Maximum withdrawals per time period
    pub max_withdrawals_per_period: u32,
    /// Time period for withdrawal rate limiting (seconds)
    pub rate_limit_period: u64,
    /// Grace period for maintaining peak tier (seconds)
    pub tier_grace_period: u64,
    /// Noise factor for probabilistic tier assignment (basis points)
    pub tier_noise_factor: u32,
    /// Whether mixer is active
    pub is_active: bool,
}

/// Mixer statistics for public view
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MixerStats {
    /// Total number of participating indexes
    pub total_indexes: u32,
    /// Total USD value in mixer
    pub total_usd_value: u128,
    /// Number of completed deposits
    pub total_deposits: u32,
    /// Number of completed withdrawals
    pub total_withdrawals: u32,
    /// Distribution of indexes by tier
    pub tier_distribution: Vec<(PortfolioTier, u32)>,
    /// Average withdrawal delay
    pub avg_withdrawal_delay: u64,
}

impl PortfolioTier {
    /// Get the minimum USD value for this tier
    pub fn min_usd_value(&self) -> u128 {
        match self {
            PortfolioTier::Micro => 10_000,
            PortfolioTier::Small => 50_000,
            PortfolioTier::Medium => 250_000,
            PortfolioTier::Large => 1_000_000,
            PortfolioTier::Whale => 5_000_000,
            PortfolioTier::Megalodon => 25_000_000,
        }
    }

    /// Get the maximum USD value for this tier
    pub fn max_usd_value(&self) -> u128 {
        match self {
            PortfolioTier::Micro => 50_000,
            PortfolioTier::Small => 250_000,
            PortfolioTier::Medium => 1_000_000,
            PortfolioTier::Large => 5_000_000,
            PortfolioTier::Whale => 25_000_000,
            PortfolioTier::Megalodon => u128::MAX,
        }
    }

    /// Get tier level for comparison (higher number = higher tier)
    pub fn level(&self) -> u32 {
        match self {
            PortfolioTier::Micro => 1,
            PortfolioTier::Small => 2,
            PortfolioTier::Medium => 3,
            PortfolioTier::Large => 4,
            PortfolioTier::Whale => 5,
            PortfolioTier::Megalodon => 6,
        }
    }
}

impl IndexCredits {
    /// Create new credits for an index
    pub fn new(initial_tier: PortfolioTier, tier_allowance: u128, timestamp: u64) -> Self {
        Self {
            current_tier: initial_tier.clone(),
            peak_tier: initial_tier,
            tier_allowance,
            used_allowance: 0,
            peak_tier_timestamp: timestamp,
            tier_decay_timestamp: timestamp,
            deposit_count: 0,
            withdrawal_count: 0,
            cumulative_deposits_usd: 0,
            is_active: true,
        }
    }

    /// Get available withdrawal allowance
    pub fn available_allowance(&self) -> u128 {
        if self.tier_allowance > self.used_allowance {
            self.tier_allowance - self.used_allowance
        } else {
            0
        }
    }

    /// Check if index can withdraw specified amount
    pub fn can_withdraw(&self, usd_amount: u128) -> bool {
        self.is_active && usd_amount <= self.available_allowance()
    }
}

/// Batch deposit request for enhanced privacy
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchDepositRequest {
    pub index_address: Address,
    pub asset_amounts: Vec<(Symbol, u128)>,
    pub total_usd_value: u128,
}

/// Result of a batch withdrawal operation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchWithdrawalResult {
    pub request_id: u32,
    pub success: bool,
    pub allocations: Vec<AssetAllocation>,
    pub error_code: Option<u32>,
}