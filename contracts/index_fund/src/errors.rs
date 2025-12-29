use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum IndexFundError {
    #[doc = "IndexFundError"]
    PathIsEmpty = 29,
    IndexMintKilled = 30,
    IndexRedeemKilled = 31,
    IndexRebalanceKilled = 32,

    // Revenue Share Errors
    ManagerNotSet = 33,
    ProtocolRecipientNotSet = 34,

    InvalidSharesDetected = 35,

    // Rebalancing Errors
    RebalanceTooSoon = 37,
    UnauthorizedRebalance = 38,
    PublicRebalanceRequiresProposal = 39,
    InvalidWeightSum = 40,
    ComponentNotFound = 41,
    InvalidComponentAction = 42,
    RebalanceNotAllowed = 46,

    // Refactoring Errors
    UnauthorizedRefactor = 45,

    NotWhitelisted = 43,
    Blacklisted = 44,

    // Redemption Errors
    InvalidAmount = 47,
    InsufficientBalance = 48,
}
