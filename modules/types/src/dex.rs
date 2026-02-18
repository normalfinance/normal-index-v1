use soroban_sdk::{contracttype, Address, String, Vec};

// FROM SOROSWAP
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexDistribution {
    pub protocol_id: String,
    pub path: Vec<Address>,
    pub parts: u32,
}
