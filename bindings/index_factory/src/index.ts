import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}





export interface FactoryConfig {
  index_contract_wasm: Buffer;
  max_manager_fee_amount: u128;
  minimum_fee_threshold: u128;
  protocol_fee_amount: u128;
  protocol_fee_recipient: string;
  swap_utility: string;
}


export interface HistoricalOracleData {
  last_oracle_price: u128;
  last_oracle_price_twap: u128;
  last_oracle_price_twap_ts: u64;
}

export type OracleValidity = {tag: "NonPositive", values: void} | {tag: "TooVolatile", values: void} | {tag: "StaleForPool", values: void} | {tag: "Frozen", values: void} | {tag: "Valid", values: void};


export interface DexDistribution {
  parts: u32;
  path: Array<string>;
  protocol_id: string;
}


export interface FeeTierConfig {
  tier_rates: Map<u128, u32>;
}


export interface UserVolumeEntry {
  index_address: string;
  timestamp: u64;
  usd_amount: u128;
}


export interface UserTierData {
  current_fee_rate_bps: u32;
  current_tier_threshold: u128;
  last_calculated: u64;
  last_volume_update: u64;
  total_30_day_volume: u128;
}

export const AccessControlError = {
  /**
   * AccessControlError
   */
  101: {message:"RoleNotFound"},
  102: {message:"Unauthorized"},
  103: {message:"AdminAlreadySet"},
  104: {message:"BadRoleUsage"},
  2906: {message:"AnotherActionActive"},
  2907: {message:"NoActionActive"},
  2908: {message:"ActionNotReadyYet"}
}

export const UpgradeError = {
  /**
   * UpgradeError
   */
  2906: {message:"AnotherActionActive"},
  2907: {message:"NoActionActive"},
  2908: {message:"ActionNotReadyYet"}
}

export const MathError = {
  /**
   * MathError: NumberOverflow
   */
  510: {message:"NumberOverflow"},
  511: {message:"MathError"}
}

export const StorageError = {
  /**
   * StorageError
   */
  501: {message:"ValueNotInitialized"},
  502: {message:"ValueMissing"}
}

export const ValidationError = {
  /**
   * ValidationError
   */
  801: {message:"InvalidToken"}
}


export interface PrivilegedAddresses {
  emergency_admin: string;
  emergency_pause_admins: Array<string>;
  operations_admin: string;
  pause_admin: string;
  rewards_admin: string;
}


export interface IndexParams {
  admin: string;
  base_nav: u128;
  blacklist_accounts: Array<string>;
  components: Array<string>;
  description: string;
  initial_deposit: u128;
  initial_price: u128;
  manager_fee_amount: u128;
  name: string;
  public: boolean;
  rebalance_authorities: Array<string>;
  token_symbol: string;
  whitelist_accounts: Array<string>;
}

export interface Client {
  /**
   * Construct and simulate a deploy_index_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deploy_index_contract: ({serialized_asset, params}: {serialized_asset: Buffer, params: IndexParams}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_protocol_fee_recipient transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_protocol_fee_recipient: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_factory_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_factory_config: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<FactoryConfig>>

  /**
   * Construct and simulate a get_swap_utility transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_swap_utility: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_protocol_fee_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_protocol_fee_amount: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_index_fee_enabled transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_index_fee_enabled: ({index_address}: {index_address: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_max_manager_fee_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_max_manager_fee_amount: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_minimum_fee_threshold transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_minimum_fee_threshold: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_index_contract_wasm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_index_contract_wasm: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Buffer>>

  /**
   * Construct and simulate a get_deployed_indexes transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_deployed_indexes: ({operator}: {operator: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_all_deployed_indexes transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_deployed_indexes: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_index_count transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_index_count: ({operator}: {operator: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_total_index_count transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_index_count: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a set_index_contract_wasm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_index_contract_wasm: ({admin, index_contract_wasm}: {admin: string, index_contract_wasm: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_protocol_fee_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_protocol_fee_amount: ({admin, amount}: {admin: string, amount: u128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_protocol_fee_recipient transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_protocol_fee_recipient: ({admin, recipient}: {admin: string, recipient: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_max_manager_fee_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_max_manager_fee_amount: ({admin, amount}: {admin: string, amount: u128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_minimum_fee_threshold transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_minimum_fee_threshold: ({admin, threshold}: {admin: string, threshold: u128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_index_fee_enabled transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_index_fee_enabled: ({admin, index_address, enabled}: {admin: string, index_address: string, enabled: boolean}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a batch_set_index_fee_enabled transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  batch_set_index_fee_enabled: ({admin, index_settings}: {admin: string, index_settings: Array<readonly [string, boolean]>}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a kill_create transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  kill_create: ({admin}: {admin: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a unkill_create transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unkill_create: ({admin}: {admin: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_is_killed_create transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_is_killed_create: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a set_oracle_registry transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_oracle_registry: ({admin, oracle_registry}: {admin: string, oracle_registry: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_oracle_registry transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_oracle_registry: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a convert_token_to_usd transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  convert_token_to_usd: ({token, amount}: {token: string, amount: u128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a set_fee_tier_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_fee_tier_config: ({admin, tier_rates}: {admin: string, tier_rates: Map<u128, u32>}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_fee_tier_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_fee_tier_config: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<FeeTierConfig>>

  /**
   * Construct and simulate a record_user_volume transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  record_user_volume: ({user, usd_amount, index_address}: {user: string, usd_amount: u128, index_address: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_user_fee_rate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_fee_rate: ({user}: {user: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_user_tier_data transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_tier_data: ({user}: {user: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<UserTierData>>

  /**
   * Construct and simulate a get_user_30_day_volume transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_30_day_volume: ({user}: {user: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  version: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a commit_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  commit_upgrade: ({admin, new_wasm_hash}: {admin: string, new_wasm_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a apply_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_upgrade: ({admin}: {admin: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Buffer>>

  /**
   * Construct and simulate a revert_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  revert_upgrade: ({admin}: {admin: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_emergency_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_emergency_mode: ({emergency_admin, value}: {emergency_admin: string, value: boolean}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_emergency_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_emergency_mode: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a commit_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  commit_transfer_ownership: ({admin, role_name, new_address}: {admin: string, role_name: string, new_address: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a apply_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_transfer_ownership: ({admin, role_name}: {admin: string, role_name: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a revert_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  revert_transfer_ownership: ({admin, role_name}: {admin: string, role_name: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_future_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_future_address: ({role_name}: {role_name: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, emergency_admin, swap_utility, index_contract_wasm, max_manager_fee_amount, protocol_fee_amount, protocol_fee_recipient, minimum_fee_threshold}: {admin: string, emergency_admin: string, swap_utility: string, index_contract_wasm: Buffer, max_manager_fee_amount: u128, protocol_fee_amount: u128, protocol_fee_recipient: string, minimum_fee_threshold: u128},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admin, emergency_admin, swap_utility, index_contract_wasm, max_manager_fee_amount, protocol_fee_amount, protocol_fee_recipient, minimum_fee_threshold}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAADUZhY3RvcnlDb25maWcAAAAAAAAGAAAAAAAAABNpbmRleF9jb250cmFjdF93YXNtAAAAA+4AAAAgAAAAAAAAABZtYXhfbWFuYWdlcl9mZWVfYW1vdW50AAAAAAAKAAAAAAAAABVtaW5pbXVtX2ZlZV90aHJlc2hvbGQAAAAAAAAKAAAAAAAAABNwcm90b2NvbF9mZWVfYW1vdW50AAAAAAoAAAAAAAAAFnByb3RvY29sX2ZlZV9yZWNpcGllbnQAAAAAABMAAAAAAAAADHN3YXBfdXRpbGl0eQAAABM=",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAgAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAPZW1lcmdlbmN5X2FkbWluAAAAABMAAAAAAAAADHN3YXBfdXRpbGl0eQAAABMAAAAAAAAAE2luZGV4X2NvbnRyYWN0X3dhc20AAAAD7gAAACAAAAAAAAAAFm1heF9tYW5hZ2VyX2ZlZV9hbW91bnQAAAAAAAoAAAAAAAAAE3Byb3RvY29sX2ZlZV9hbW91bnQAAAAACgAAAAAAAAAWcHJvdG9jb2xfZmVlX3JlY2lwaWVudAAAAAAAEwAAAAAAAAAVbWluaW11bV9mZWVfdGhyZXNob2xkAAAAAAAACgAAAAA=",
        "AAAAAAAAAAAAAAAVZGVwbG95X2luZGV4X2NvbnRyYWN0AAAAAAAAAgAAAAAAAAAQc2VyaWFsaXplZF9hc3NldAAAAA4AAAAAAAAABnBhcmFtcwAAAAAH0AAAAAtJbmRleFBhcmFtcwAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAaZ2V0X3Byb3RvY29sX2ZlZV9yZWNpcGllbnQAAAAAAAAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAASZ2V0X2ZhY3RvcnlfY29uZmlnAAAAAAAAAAAAAQAAB9AAAAANRmFjdG9yeUNvbmZpZwAAAA==",
        "AAAAAAAAAAAAAAAQZ2V0X3N3YXBfdXRpbGl0eQAAAAAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAXZ2V0X3Byb3RvY29sX2ZlZV9hbW91bnQAAAAAAAAAAAEAAAAK",
        "AAAAAAAAAAAAAAAVZ2V0X2luZGV4X2ZlZV9lbmFibGVkAAAAAAAAAQAAAAAAAAANaW5kZXhfYWRkcmVzcwAAAAAAABMAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAaZ2V0X21heF9tYW5hZ2VyX2ZlZV9hbW91bnQAAAAAAAAAAAABAAAACg==",
        "AAAAAAAAAAAAAAAZZ2V0X21pbmltdW1fZmVlX3RocmVzaG9sZAAAAAAAAAAAAAABAAAACg==",
        "AAAAAAAAAAAAAAAXZ2V0X2luZGV4X2NvbnRyYWN0X3dhc20AAAAAAAAAAAEAAAPuAAAAIA==",
        "AAAAAAAAAAAAAAAUZ2V0X2RlcGxveWVkX2luZGV4ZXMAAAABAAAAAAAAAAhvcGVyYXRvcgAAABMAAAABAAAD6gAAABM=",
        "AAAAAAAAAAAAAAAYZ2V0X2FsbF9kZXBsb3llZF9pbmRleGVzAAAAAAAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAPZ2V0X2luZGV4X2NvdW50AAAAAAEAAAAAAAAACG9wZXJhdG9yAAAAEwAAAAEAAAAE",
        "AAAAAAAAAAAAAAAVZ2V0X3RvdGFsX2luZGV4X2NvdW50AAAAAAAAAAAAAAEAAAAE",
        "AAAAAAAAAAAAAAAXc2V0X2luZGV4X2NvbnRyYWN0X3dhc20AAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABNpbmRleF9jb250cmFjdF93YXNtAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAAXc2V0X3Byb3RvY29sX2ZlZV9hbW91bnQAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAoAAAAA",
        "AAAAAAAAAAAAAAAac2V0X3Byb3RvY29sX2ZlZV9yZWNpcGllbnQAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcmVjaXBpZW50AAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAac2V0X21heF9tYW5hZ2VyX2ZlZV9hbW91bnQAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAAKAAAAAA==",
        "AAAAAAAAAAAAAAAZc2V0X21pbmltdW1fZmVlX3RocmVzaG9sZAAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJdGhyZXNob2xkAAAAAAAACgAAAAA=",
        "AAAAAAAAAAAAAAAVc2V0X2luZGV4X2ZlZV9lbmFibGVkAAAAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAA1pbmRleF9hZGRyZXNzAAAAAAAAEwAAAAAAAAAHZW5hYmxlZAAAAAABAAAAAA==",
        "AAAAAAAAAAAAAAAbYmF0Y2hfc2V0X2luZGV4X2ZlZV9lbmFibGVkAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAOaW5kZXhfc2V0dGluZ3MAAAAAA+oAAAPtAAAAAgAAABMAAAABAAAAAA==",
        "AAAAAAAAAAAAAAALa2lsbF9jcmVhdGUAAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAANdW5raWxsX2NyZWF0ZQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAUZ2V0X2lzX2tpbGxlZF9jcmVhdGUAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAATc2V0X29yYWNsZV9yZWdpc3RyeQAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAD29yYWNsZV9yZWdpc3RyeQAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAATZ2V0X29yYWNsZV9yZWdpc3RyeQAAAAAAAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAUY29udmVydF90b2tlbl90b191c2QAAAACAAAAAAAAAAV0b2tlbgAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACgAAAAEAAAAK",
        "AAAAAAAAAAAAAAATc2V0X2ZlZV90aWVyX2NvbmZpZwAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAACnRpZXJfcmF0ZXMAAAAAA+wAAAAKAAAABAAAAAA=",
        "AAAAAAAAAAAAAAATZ2V0X2ZlZV90aWVyX2NvbmZpZwAAAAAAAAAAAQAAB9AAAAANRmVlVGllckNvbmZpZwAAAA==",
        "AAAAAAAAAAAAAAAScmVjb3JkX3VzZXJfdm9sdW1lAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAKdXNkX2Ftb3VudAAAAAAACgAAAAAAAAANaW5kZXhfYWRkcmVzcwAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAARZ2V0X3VzZXJfZmVlX3JhdGUAAAAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAAE",
        "AAAAAAAAAAAAAAASZ2V0X3VzZXJfdGllcl9kYXRhAAAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAfQAAAADFVzZXJUaWVyRGF0YQ==",
        "AAAAAAAAAAAAAAAWZ2V0X3VzZXJfMzBfZGF5X3ZvbHVtZQAAAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAACg==",
        "AAAAAAAAAAAAAAAHdmVyc2lvbgAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAOY29tbWl0X3VwZ3JhZGUAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAANYXBwbHlfdXBncmFkZQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPuAAAAIA==",
        "AAAAAAAAAAAAAAAOcmV2ZXJ0X3VwZ3JhZGUAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAASc2V0X2VtZXJnZW5jeV9tb2RlAAAAAAACAAAAAAAAAA9lbWVyZ2VuY3lfYWRtaW4AAAAAEwAAAAAAAAAFdmFsdWUAAAAAAAABAAAAAA==",
        "AAAAAAAAAAAAAAASZ2V0X2VtZXJnZW5jeV9tb2RlAAAAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAZY29tbWl0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAAAAAALbmV3X2FkZHJlc3MAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAYYXBwbHlfdHJhbnNmZXJfb3duZXJzaGlwAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAA==",
        "AAAAAAAAAAAAAAAZcmV2ZXJ0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAA=",
        "AAAAAAAAAAAAAAASZ2V0X2Z1dHVyZV9hZGRyZXNzAAAAAAABAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAQAAABM=",
        "AAAAAQAAAAAAAAAAAAAAFEhpc3RvcmljYWxPcmFjbGVEYXRhAAAAAwAAAAAAAAARbGFzdF9vcmFjbGVfcHJpY2UAAAAAAAAKAAAAAAAAABZsYXN0X29yYWNsZV9wcmljZV90d2FwAAAAAAAKAAAAAAAAABlsYXN0X29yYWNsZV9wcmljZV90d2FwX3RzAAAAAAAABg==",
        "AAAAAgAAAAAAAAAAAAAADk9yYWNsZVZhbGlkaXR5AAAAAAAFAAAAAAAAAAAAAAALTm9uUG9zaXRpdmUAAAAAAAAAAAAAAAALVG9vVm9sYXRpbGUAAAAAAAAAAAAAAAAMU3RhbGVGb3JQb29sAAAAAAAAAAAAAAAGRnJvemVuAAAAAAAAAAAAAAAAAAVWYWxpZAAAAA==",
        "AAAAAQAAAAAAAAAAAAAAD0RleERpc3RyaWJ1dGlvbgAAAAADAAAAAAAAAAVwYXJ0cwAAAAAAAAQAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAALcHJvdG9jb2xfaWQAAAAAEA==",
        "AAAAAQAAAAAAAAAAAAAADUZlZVRpZXJDb25maWcAAAAAAAABAAAAAAAAAAp0aWVyX3JhdGVzAAAAAAPsAAAACgAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAD1VzZXJWb2x1bWVFbnRyeQAAAAADAAAAAAAAAA1pbmRleF9hZGRyZXNzAAAAAAAAEwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABgAAAAAAAAAKdXNkX2Ftb3VudAAAAAAACg==",
        "AAAAAQAAAAAAAAAAAAAADFVzZXJUaWVyRGF0YQAAAAUAAAAAAAAAFGN1cnJlbnRfZmVlX3JhdGVfYnBzAAAABAAAAAAAAAAWY3VycmVudF90aWVyX3RocmVzaG9sZAAAAAAACgAAAAAAAAAPbGFzdF9jYWxjdWxhdGVkAAAAAAYAAAAAAAAAEmxhc3Rfdm9sdW1lX3VwZGF0ZQAAAAAABgAAAAAAAAATdG90YWxfMzBfZGF5X3ZvbHVtZQAAAAAK",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAABwAAABJBY2Nlc3NDb250cm9sRXJyb3IAAAAAAAxSb2xlTm90Rm91bmQAAABlAAAAAAAAAAxVbmF1dGhvcml6ZWQAAABmAAAAAAAAAA9BZG1pbkFscmVhZHlTZXQAAAAAZwAAAAAAAAAMQmFkUm9sZVVzYWdlAAAAaAAAAAAAAAATQW5vdGhlckFjdGlvbkFjdGl2ZQAAAAtaAAAAAAAAAA5Ob0FjdGlvbkFjdGl2ZQAAAAALWwAAAAAAAAARQWN0aW9uTm90UmVhZHlZZXQAAAAAAAtc",
        "AAAABAAAAAAAAAAAAAAADFVwZ3JhZGVFcnJvcgAAAAMAAAAMVXBncmFkZUVycm9yAAAAE0Fub3RoZXJBY3Rpb25BY3RpdmUAAAALWgAAAAAAAAAOTm9BY3Rpb25BY3RpdmUAAAAAC1sAAAAAAAAAEUFjdGlvbk5vdFJlYWR5WWV0AAAAAAALXA==",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAIAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAAAAAAAJTWF0aEVycm9yAAAAAAAB/w==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAIAAAAMU3RvcmFnZUVycm9yAAAAE1ZhbHVlTm90SW5pdGlhbGl6ZWQAAAAB9QAAAAAAAAAMVmFsdWVNaXNzaW5nAAAB9g==",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAABAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQ==",
        "AAAAAQAAAAAAAAAAAAAAE1ByaXZpbGVnZWRBZGRyZXNzZXMAAAAABQAAAAAAAAAPZW1lcmdlbmN5X2FkbWluAAAAABMAAAAAAAAAFmVtZXJnZW5jeV9wYXVzZV9hZG1pbnMAAAAAA+oAAAATAAAAAAAAABBvcGVyYXRpb25zX2FkbWluAAAAEwAAAAAAAAALcGF1c2VfYWRtaW4AAAAAEwAAAAAAAAANcmV3YXJkc19hZG1pbgAAAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAAC0luZGV4UGFyYW1zAAAAAA0AAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIYmFzZV9uYXYAAAAKAAAAAAAAABJibGFja2xpc3RfYWNjb3VudHMAAAAAA+oAAAATAAAAAAAAAApjb21wb25lbnRzAAAAAAPqAAAAEwAAAAAAAAALZGVzY3JpcHRpb24AAAAAEAAAAAAAAAAPaW5pdGlhbF9kZXBvc2l0AAAAAAoAAAAAAAAADWluaXRpYWxfcHJpY2UAAAAAAAAKAAAAAAAAABJtYW5hZ2VyX2ZlZV9hbW91bnQAAAAAAAoAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAZwdWJsaWMAAAAAAAEAAAAAAAAAFXJlYmFsYW5jZV9hdXRob3JpdGllcwAAAAAAA+oAAAATAAAAAAAAAAx0b2tlbl9zeW1ib2wAAAAQAAAAAAAAABJ3aGl0ZWxpc3RfYWNjb3VudHMAAAAAA+oAAAAT" ]),
      options
    )
  }
  public readonly fromJSON = {
    deploy_index_contract: this.txFromJSON<string>,
        get_protocol_fee_recipient: this.txFromJSON<string>,
        get_factory_config: this.txFromJSON<FactoryConfig>,
        get_swap_utility: this.txFromJSON<string>,
        get_protocol_fee_amount: this.txFromJSON<u128>,
        get_index_fee_enabled: this.txFromJSON<boolean>,
        get_max_manager_fee_amount: this.txFromJSON<u128>,
        get_minimum_fee_threshold: this.txFromJSON<u128>,
        get_index_contract_wasm: this.txFromJSON<Buffer>,
        get_deployed_indexes: this.txFromJSON<Array<string>>,
        get_all_deployed_indexes: this.txFromJSON<Array<string>>,
        get_index_count: this.txFromJSON<u32>,
        get_total_index_count: this.txFromJSON<u32>,
        set_index_contract_wasm: this.txFromJSON<null>,
        set_protocol_fee_amount: this.txFromJSON<null>,
        set_protocol_fee_recipient: this.txFromJSON<null>,
        set_max_manager_fee_amount: this.txFromJSON<null>,
        set_minimum_fee_threshold: this.txFromJSON<null>,
        set_index_fee_enabled: this.txFromJSON<null>,
        batch_set_index_fee_enabled: this.txFromJSON<null>,
        kill_create: this.txFromJSON<null>,
        unkill_create: this.txFromJSON<null>,
        get_is_killed_create: this.txFromJSON<boolean>,
        set_oracle_registry: this.txFromJSON<null>,
        get_oracle_registry: this.txFromJSON<string>,
        convert_token_to_usd: this.txFromJSON<u128>,
        set_fee_tier_config: this.txFromJSON<null>,
        get_fee_tier_config: this.txFromJSON<FeeTierConfig>,
        record_user_volume: this.txFromJSON<null>,
        get_user_fee_rate: this.txFromJSON<u32>,
        get_user_tier_data: this.txFromJSON<UserTierData>,
        get_user_30_day_volume: this.txFromJSON<u128>,
        version: this.txFromJSON<u32>,
        commit_upgrade: this.txFromJSON<null>,
        apply_upgrade: this.txFromJSON<Buffer>,
        revert_upgrade: this.txFromJSON<null>,
        set_emergency_mode: this.txFromJSON<null>,
        get_emergency_mode: this.txFromJSON<boolean>,
        commit_transfer_ownership: this.txFromJSON<null>,
        apply_transfer_ownership: this.txFromJSON<null>,
        revert_transfer_ownership: this.txFromJSON<null>,
        get_future_address: this.txFromJSON<string>
  }
}