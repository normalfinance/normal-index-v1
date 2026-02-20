use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
/// Index fund contract error codes.
pub enum IndexFundError {
    /// Swap path generation failed validation.
    PathIsEmpty = 29,
    /// Minting is disabled by emergency controls.
    IndexMintKilled = 30,
    /// Redemption is disabled by emergency controls.
    IndexRedeemKilled = 31,
    /// Rebalancing is disabled by emergency controls.
    IndexRebalanceKilled = 32,

    // Revenue Share Errors
    /// Manager recipient is not configured.
    ManagerNotSet = 33,
    /// Protocol recipient is not configured.
    ProtocolRecipientNotSet = 34,

    /// Share math inputs are inconsistent.
    InvalidSharesDetected = 35,

    // Rebalancing Errors
    /// Rebalance attempted before cooldown elapsed.
    RebalanceTooSoon = 37,
    /// Caller is not authorized to rebalance.
    UnauthorizedRebalance = 38,
    /// Public rebalance requires a proposal flow.
    PublicRebalanceRequiresProposal = 39,
    /// Component weights do not sum to 10_000 bps.
    InvalidWeightSum = 40,
    /// Target component does not exist.
    ComponentNotFound = 41,
    /// Component action is malformed or unsupported.
    InvalidComponentAction = 42,
    /// Rebalance is currently disallowed.
    RebalanceNotAllowed = 46,

    // Refactoring Errors
    /// Caller is not authorized to refactor components.
    UnauthorizedRefactor = 45,

    /// Address is not whitelisted for private index operations.
    NotWhitelisted = 43,
    /// Address is blacklisted from index operations.
    Blacklisted = 44,

    // Redemption Errors
    /// Amount must be greater than zero.
    InvalidAmount = 47,
    /// Balance is insufficient for requested action.
    InsufficientBalance = 48,

    // Oracle Errors
    /// Component has no configured oracle.
    MissingOracleAddress = 49,

    /// Adapter lookup failed in the adapter registry.
    FailedToGetAdapter = 50,
}
