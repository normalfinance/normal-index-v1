use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum MixerError {
    /// Mixer is not initialized
    NotInitialized = 1,
    /// Mixer is not active
    NotActive = 2,
    /// Index is not authorized to use mixer
    UnauthorizedIndex = 3,
    /// Insufficient tier allowance for withdrawal
    InsufficientTierAllowance = 4,
    /// Withdrawal amount exceeds available balance
    InsufficientMixerBalance = 5,
    /// Minimum anonymity set not reached
    InsufficientAnonymitySet = 6,
    /// Withdrawal request not found
    WithdrawalRequestNotFound = 7,
    /// Withdrawal request is not pending
    WithdrawalNotPending = 8,
    /// Withdrawal delay period not elapsed
    WithdrawalTooEarly = 9,
    /// Rate limit exceeded for withdrawals
    WithdrawalRateLimitExceeded = 10,
    /// Invalid withdrawal amount (zero or too large)
    InvalidWithdrawalAmount = 11,
    /// Invalid deposit amount (zero or too large)
    InvalidDepositAmount = 12,
    /// Asset not supported by mixer
    UnsupportedAsset = 13,
    /// Invalid tier configuration
    InvalidTierConfig = 14,
    /// Index already has pending withdrawal
    PendingWithdrawalExists = 15,
    /// Cannot withdraw more than deposited
    WithdrawalExceedsDeposits = 16,
    /// Invalid preferred assets list
    InvalidPreferredAssets = 17,
    /// Price oracle not available
    PriceOracleUnavailable = 18,
    /// Calculation overflow
    CalculationOverflow = 19,
    /// Index not found in mixer
    IndexNotFound = 20,
}