use soroban_sdk::{Address, Env, IntoVal, Symbol};
use crate::storage::get_factory_safe;


pub struct VolumeTracker;

impl VolumeTracker {
    
    
    pub fn record_mint_volume(env: &Env, user: &Address, token: &Address, amount: u128) {
        if let Some(usd_amount) = Self::convert_to_usd(env, token, amount) {
            Self::record_user_volume_in_factory(env, user, usd_amount);
        }
    }
    
    
    pub fn record_redeem_volume(env: &Env, user: &Address, share_value_usd: u128) {
        Self::record_user_volume_in_factory(env, user, share_value_usd);
    }
    
    
    fn record_user_volume_in_factory(env: &Env, user: &Address, usd_amount: u128) {
        if let Some(factory_address) = get_factory_safe(env) {
            
            let _result = env.try_invoke_contract::<(), soroban_sdk::Error>(
                &factory_address,
                &Symbol::new(env, "record_user_volume"),
                soroban_sdk::Vec::from_array(env, [
                    user.clone().into_val(env),
                    usd_amount.into_val(env),
                    env.current_contract_address().into_val(env),
                ]),
            );
    
        }
    }
    
    
    
    fn convert_to_usd(env: &Env, token: &Address, amount: u128) -> Option<u128> {
        let factory_address = get_factory_safe(env)?;
        
        //should be implemented in the factory contract
        None
    }
    
    
    
    pub fn calculate_redeem_usd_value(env: &Env, shares: u128, nav_per_share: u128) -> u128 {
        //TODO: implement this in the factory contract, for now we'll just return the shares * nav_per_share
        shares.saturating_mul(nav_per_share).saturating_div(10_000_000) 
    }
}