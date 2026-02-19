use soroban_sdk::{Address, Env, Map, Symbol};

pub trait AdapterRegistryTrait {
    fn set_adapter(e: Env, admin: Address, name: Symbol, adapter: Address);
    fn get_adapter(e: Env, name: Symbol) -> Address;
    fn get_adapter_safe(e: Env, name: Symbol) -> Option<Address>;
    fn get_adapter_name(e: Env, adapter: Address) -> Symbol;
    fn get_adapters(e: Env) -> Map<Symbol, Address>;
}
