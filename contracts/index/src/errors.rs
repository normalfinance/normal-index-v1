use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum IndexError {
    #[doc = "IndexError"]
    MaxIFWithdrawReached = 0,

    PathIsEmpty = 29,
    IndexMintKilled = 30,
    IndexRedeemKilled = 31,
    IndexRebalanceKilled = 32,

    // Revenue Share Errors
    ManagerNotSet = 33,
    ProtocolRecipientNotSet = 34,

    // Insurance Fund Errors
    InvalidIFForNewStakes = 35,
    InvalidIFSharesDetected = 36,
    
    // Rebalancing Errors
    RebalanceTooSoon = 37,
    UnauthorizedRebalance = 38,
    PublicRebalanceRequiresProposal = 39,
    InvalidWeightSum = 40,
    ComponentNotFound = 41,
    InvalidComponentAction = 42,
}
