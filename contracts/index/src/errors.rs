use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum IndexError {
    #[doc = "IndexError"]
    MaxIFWithdrawReached = 0,
   

    IndexMintKilled = 30,
    IndexRedeemKilled = 31,
    IndexRebalanceKilled = 32,
}
