use adapter::AdapterError;
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    vec, Address, Env, IntoVal, Symbol,
};

soroban_sdk::contractimport!(file = "../../external_wasms/normal/treasury.wasm");
pub type NormalTreasuryClient<'a> = Client<'a>;

// Define the RequestType enum with explicit u32 values
#[derive(Clone, PartialEq)]
#[repr(u32)]
pub enum RequestType {
    BuyLong = 0,
    BuyShort = 1,
    SellLong = 2,
    SellShort = 3,
}

// Implement a method to convert RequestType to u32
impl RequestType {
    fn to_u32(self) -> u32 {
        self as u32
    }
}

pub fn buy_long(
    e: &Env,
    user: &Address,
    pair: &Address,
    amount: &u128,
) -> Result<u128, AdapterError> {
    let treasury_client = NormalTreasuryClient::new(e, &crate::storage::get_treasury(e));

    e.authorize_as_current_contract(vec![
        &e,
        InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: pair.clone(),
                fn_name: Symbol::new(&e, "transfer"),
                args: (e.current_contract_address(), pair.clone(), amount.clone()).into_val(e),
            },
            sub_invocations: vec![&e],
        }),
    ]);

    treasury_client.buy_long(user, pair, amount, &0);

    Ok(*amount)
}
