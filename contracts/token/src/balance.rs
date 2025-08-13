use crate::errors::TokenError;
use soroban_sdk::{contracttype, panic_with_error, Address, Env};
use utils::bump::bump_persistent;

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Balance(Address),
}

fn write_balance(e: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr);
    e.storage().persistent().set(&key, &amount);
    bump_persistent(e, &key);
}

pub fn read_balance(e: &Env, addr: Address) -> i128 {
    //condition to check if the parent index is private, if yes, then we can't read the balance
    let key = DataKey::Balance(addr);
    match e.storage().persistent().get::<DataKey, i128>(&key) {
        Some(balance) => {
            bump_persistent(e, &key);
            balance
        }
        None => 0,
    }
}

pub fn receive_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_balance(e, addr.clone());
    write_balance(e, addr, balance + amount);
}

pub fn spend_balance(e: &Env, addr: Address, amount: i128) {
    let balance = read_balance(e, addr.clone());
    if balance < amount {
        panic_with_error!(&e, TokenError::InsufficientBalance);
    }
    write_balance(e, addr, balance - amount);
}
