use soroban_sdk::{Address, BytesN, Env, Map, Vec};
use utils::storage::IndexParams;

use crate::contract::FactoryConfig;

pub trait IndexFactoryTrait {
    //  ___      ___       __        __    _____  ___
    // |"  \    /"  |     /""\      |" \  (\"   \|"  \
    //  \   \  //   |    /    \     ||  | |.\\   \    |
    //  /\\  \/.    |   /' /\  \    |:  | |: \.   \\  |
    // |: \.        |  //  __'  \   |.  | |.  \    \. |
    // |.  \    /:  | /   /  \\  \  /\  |\|    \    \ |
    // |___|\__/|___|(___/    \___)(__\_|_)\___|\____\)

    fn deploy_index_contract(e: Env, params: IndexParams) -> Address;
}

pub trait AdminInterface {
    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    fn get_protocol_fee_recipient(e: Env) -> Address;

    fn get_factory_config(e: Env) -> FactoryConfig;

    fn get_swap_utility(e: Env) -> Address;

    fn get_protocol_fee_amount(e: Env) -> u128;

    fn get_max_manager_fee_amount(e: Env) -> u128;

    fn get_minimum_fee_threshold(e: Env) -> u128;

    fn get_index_fee_enabled(e: Env, index_address: Address) -> bool;

    fn get_index_contract_wasm(e: Env) -> BytesN<32>;

    fn get_token_contract_wasm(e: Env) -> BytesN<32>;

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

    fn set_token_contract_wasm(e: Env, admin: Address, token_contract_wasm: BytesN<32>);

    fn set_protocol_fee_amount(e: Env, admin: Address, amount: u128);

    fn set_protocol_fee_recipient(e: Env, admin: Address, recipient: Address);

    fn set_max_manager_fee_amount(e: Env, admin: Address, amount: u128);

    fn set_minimum_fee_threshold(e: Env, admin: Address, threshold: u128);

    fn set_index_fee_enabled(e: Env, admin: Address, index_address: Address, enabled: bool);

    fn batch_set_index_fee_enabled(e: Env, admin: Address, index_settings: Vec<(Address, bool)>);

    fn set_oracle_registry(e: Env, admin: Address, oracle_registry: Address);

    fn get_oracle_registry(e: Env) -> Address;

    fn convert_token_to_usd(e: Env, token: Address, amount: u128) -> u128;

    fn set_fee_tier_config(e: Env, admin: Address, tier_rates: Map<u128, u32>);

    fn get_fee_tier_config(e: Env) -> crate::storage::FeeTierConfig;

    // User-level tier methods
    fn record_user_volume(e: Env, user: Address, usd_amount: u128, index_address: Address);

    fn get_user_fee_rate(e: Env, user: Address) -> u32;

    fn get_user_tier_data(e: Env, user: Address) -> crate::storage::UserTierData;

    fn get_user_30_day_volume(e: Env, user: Address) -> u128;

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
