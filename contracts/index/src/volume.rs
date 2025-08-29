use crate::errors::IndexError;
use crate::storage::get_factory_safe;
use soroban_sdk::{Address, Env, IntoVal, Symbol};

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
                soroban_sdk::Vec::from_array(
                    env,
                    [
                        user.clone().into_val(env),
                        usd_amount.into_val(env),
                        env.current_contract_address().into_val(env),
                    ],
                ),
            );
        }
    }

    fn convert_to_usd(env: &Env, token: &Address, amount: u128) -> Option<u128> {
        let factory_address = get_factory_safe(env)?;

        let result = env.try_invoke_contract::<u128, IndexError>(
            &factory_address,
            &Symbol::new(env, "convert_token_to_usd"),
            soroban_sdk::Vec::from_array(env, [token.clone().into_val(env), amount.into_val(env)]),
        );

        match result {
            Ok(contract_result) => match contract_result {
                Ok(usd_amount) => Some(usd_amount),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    pub fn calculate_redeem_usd_value(_env: &Env, shares: u128, price_per_share: u128) -> u128 {
        shares
            .saturating_mul(price_per_share)
            .saturating_div(10_000_000)
    }
}
