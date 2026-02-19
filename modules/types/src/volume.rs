use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeFeeTier {
    pub min_monthly_volume: u128,
    pub protocol_fee_bps: u32,
    pub manager_fee_bps: u32,
}
