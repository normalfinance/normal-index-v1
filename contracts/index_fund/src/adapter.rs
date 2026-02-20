use soroban_sdk::{panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use types::adapter::AdapterTradeParams;

use crate::errors::IndexFundError;

/// Resolves an adapter contract address from the adapter registry.
///
/// Panics with [`IndexFundError::FailedToGetAdapter`] when the registry lookup fails.
///
/// # Arguments
/// - `e` (`&Env`): Soroban environment.
/// - `adapter` (`&Symbol`): Adapter name symbol to resolve.
///
/// # Returns
/// - `Address`: Resolved adapter contract address.
pub fn get_adapter_from_registry(e: &Env, adapter: &Symbol) -> Address {
    match e.try_invoke_contract::<Address, soroban_sdk::Error>(
        &crate::storage::get_adapter_registry(e),
        &Symbol::new(e, "get_adapter"),
        Vec::from_array(e, [adapter.into_val(e)]),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, IndexFundError::FailedToGetAdapter),
        Ok(Ok(adapter_address)) => {
            return adapter_address;
        }
    }
}
