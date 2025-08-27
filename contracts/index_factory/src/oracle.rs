use soroban_sdk::{contract, contractimpl, contractclient, Address, Env, Symbol, Vec, panic_with_error};


#[contractclient(name = "OracleRegistryClient")]
pub trait OracleRegistry {
    
    
    fn get_price_in_usd(env: Env, asset: Address) -> i128;
}


pub struct OracleUtils;

impl OracleUtils {
      
    
    pub fn convert_xlm_to_usd(env: &Env, oracle_registry: &Address, xlm_asset: &Address, xlm_amount: u128) -> u128 {
        
        
        0
    }
    
    
    
    pub fn get_xlm_price_usd(env: &Env, oracle_registry: &Address, xlm_asset: &Address) -> u128 {
        
        
        0
    }
}

