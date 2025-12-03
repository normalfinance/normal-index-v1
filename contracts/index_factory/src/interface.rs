use soroban_sdk::{ Address, Bytes, BytesN, Env, Map, Vec };
use utils::storage::IndexParams;

use crate::contract::FactoryConfig;

pub trait IndexFactoryTrait {
    fn deploy_index_contract(e: Env, serialized_asset: Bytes, params: IndexParams) -> Address;
}

pub trait AdminInterface {
    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    fn get_factory_config(e: Env) -> FactoryConfig;

    fn get_swap_utility(e: Env) -> Address;

    fn get_index_contract_wasm(e: Env) -> BytesN<32>;

    fn get_deployed_indexes(e: Env, operator: Address) -> Vec<Address>;

    fn get_all_deployed_indexes(e: Env) -> Vec<Address>;

    fn get_index_count(e: Env, operator: Address) -> u32;

    fn get_total_index_count(e: Env) -> u32;

    //   ________  _______  ___________  ___________  _______   _______    ________
    //  /"       )/"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (:   \___/(: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \___  \   \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //   __/  \\  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    //  /" \   :)(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    // (_______/  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    fn set_index_contract_wasm(e: Env, admin: Address, index_contract_wasm: BytesN<32>);

    fn set_oracle_registry(e: Env, admin: Address, oracle_registry: Address);

    fn get_oracle_registry(e: Env) -> Address;

    fn convert_token_to_usd(e: Env, token: Address, amount: u128) -> u128;

    //    _______     __       ____  ____   ________  _______  ________
    //   |   __ "\   /""\     ("  _||_ " | /"       )/"     "||"      "\
    //   (. |__) :) /    \    |   (  ) : |(:   \___/(: ______)(.  ___  :)
    //   |:  ____/ /' /\  \   (:  |  | . ) \___  \   \/    |  |: \   ) ||
    //   (|  /    //  __'  \   \\ \__/ //   __/  \\  // ___)_ (| (___\ ||
    //  /|__/ \  /   /  \\  \  /\\ __ //\  /" \   :)(:      "||:       :)
    // (_______)(___/    \___)(__________)(_______/  \_______)(________/

    // Stop index creation instantly
    fn kill_create(e: Env, admin: Address);

    // Resume index creation
    fn unkill_create(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_create(e: Env) -> bool;
}
