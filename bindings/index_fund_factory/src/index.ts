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


export interface DexDistribution {
  parts: u32;
  path: Array<string>;
  protocol_id: string;
}


export interface IndexParams {
  admin: string;
  components: Array<ComponentUpdate>;
  description: string;
  initial_price: u128;
  is_public: boolean;
  name: string;
  symbol: string;
  token_quote: string;
}


export interface IndexFundInfo {
  address: string;
  admin_address: string;
  initial_price: u128;
  is_public: boolean;
  last_rebalance_ts: u64;
  last_updated_ts: u64;
  rebalance_threshold: u64;
  token_address: string;
  total_mints: u128;
  total_redemptions: u128;
  total_shares: u128;
}


export interface IndexFundMetrics {
  current_nav: u128;
  share_price: u128;
  total_mints: u128;
  total_redemptions: u128;
  total_shares: u128;
}


export interface IndexFundStatus {
  can_rebalance: boolean;
  is_public: boolean;
  last_rebalance_ts: u64;
  rebalance_threshold: u64;
}


export interface VolumeFeeTier {
  manager_fee_bps: u32;
  min_monthly_volume: u128;
  protocol_fee_bps: u32;
}

export type AdapterType = {tag: "Normal", values: void} | {tag: "Aquarius", values: void} | {tag: "Soroswap", values: void};


export interface AdapterTradeParams {
  amount_in: u128;
  amount_out_min: u128;
  asset: string;
  to: string;
  token_in: string;
  token_out: string;
}


export interface AdapterResult {
  amount_in: u128;
  amount_out: u128;
  success: boolean;
}

export enum AdapterError {
  ProviderNotSupported = 100,
  ProviderNotConfigured = 101,
  InvalidTokenPair = 200,
  InvalidAmount = 201,
  InvalidSlippage = 202,
  InsufficientLiquidity = 300,
  SlippageExceeded = 301,
  SwapFailed = 302,
  NormalDexFailed = 400,
  SoroswapSwapFailed = 401,
  SoroswapAggregatorUnavailable = 402,
  InvalidProviderConfig = 500,
  UnauthorizedAccess = 501,
  ContractNotInitialized = 502,
}


export interface Component {
  adapter: string;
  adapter_type: AdapterType;
  asset: string;
  oracle: string;
  weight: u128;
}

export type ComponentAction = {tag: "Add", values: void} | {tag: "Remove", values: void} | {tag: "UpdateWeight", values: void};


export interface ComponentUpdate {
  action: ComponentAction;
  adapter: string;
  adapter_type: AdapterType;
  new_weight: u128;
  oracle: Option<string>;
  token: string;
}


export interface RefactorParams {
  component_updates: Array<ComponentUpdate>;
}


export interface RebalanceParams {
  target_nav: Option<i128>;
}


export interface ComponentAllocation {
  component: Component;
  current_balance: u128;
  percentage_of_nav: u128;
  target_balance: u128;
}


export interface RebalanceStatus {
  authorized_rebalancers: Array<string>;
  can_rebalance: boolean;
  is_public: boolean;
  last_rebalance_ts: u64;
  rebalance_threshold: u64;
  time_until_next_rebalance: u64;
}

export type OracleSource = {tag: "Reflector", values: void};

export const OracleError = {
  /**
   * OracleError: OracleNonPositive
   */
  601: {message:"OracleNonPositive"},
  602: {message:"OracleTooVolatile"},
  603: {message:"OracleStaleForPool"},
  604: {message:"OracleInvalid"},
  605: {message:"FailedToGetOraclePrice"}
}


export interface OraclePriceData {
  delay: u64;
  price: u128;
}

export type OracleValidity = {tag: "NonPositive", values: void} | {tag: "TooVolatile", values: void} | {tag: "StaleForPool", values: void} | {tag: "Frozen", values: void} | {tag: "Valid", values: void};


export interface HistoricalOracleData {
  last_price: u128;
  last_price_twap: u128;
  last_update_ts: u64;
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
   * Construct and simulate a mint transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  mint: ({user, index, amount}: {user: string, index: string, amount: u128}, options?: {
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
   * Construct and simulate a redeem transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  redeem: ({user, index, share_amount}: {user: string, index: string, share_amount: u128}, options?: {
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
   * Construct and simulate a rebalance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  rebalance: ({caller, index, params}: {caller: string, index: string, params: RebalanceParams}, options?: {
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
   * Construct and simulate a refactor transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refactor: ({caller, index, params}: {caller: string, index: string, params: RefactorParams}, options?: {
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
   * Construct and simulate a claim_system_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_system_fees: ({caller, index, token, destination}: {caller: string, index: string, token: string, destination: string}, options?: {
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
   * Construct and simulate a claim_manager_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_manager_fees: ({caller, index, token, destination}: {caller: string, index: string, token: string, destination: string}, options?: {
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
   * Construct and simulate a set_privileged_addrs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_privileged_addrs: ({admin, rewards_admin, operations_admin, fee_admin}: {admin: string, rewards_admin: string, operations_admin: string, fee_admin: string}, options?: {
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
   * Construct and simulate a get_privileged_addrs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_privileged_addrs: (options?: {
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
  }) => Promise<AssembledTransaction<Map<string, Array<string>>>>

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
   * Construct and simulate a contract_name transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  contract_name: (options?: {
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
        {admin, emergency_admin, rewards_admin, operations_admin, fee_admin, index_contract_wasm}: {admin: string, emergency_admin: string, rewards_admin: string, operations_admin: string, fee_admin: string, index_contract_wasm: Buffer},
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
    return ContractClient.deploy({admin, emergency_admin, rewards_admin, operations_admin, fee_admin, index_contract_wasm}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAYAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAPZW1lcmdlbmN5X2FkbWluAAAAABMAAAAAAAAADXJld2FyZHNfYWRtaW4AAAAAAAATAAAAAAAAABBvcGVyYXRpb25zX2FkbWluAAAAEwAAAAAAAAAJZmVlX2FkbWluAAAAAAAAEwAAAAAAAAATaW5kZXhfY29udHJhY3Rfd2FzbQAAAAPuAAAAIAAAAAA=",
        "AAAAAAAAAAAAAAAVZGVwbG95X2luZGV4X2NvbnRyYWN0AAAAAAAAAgAAAAAAAAAQc2VyaWFsaXplZF9hc3NldAAAAA4AAAAAAAAABnBhcmFtcwAAAAAH0AAAAAtJbmRleFBhcmFtcwAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAEbWludAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAVpbmRleAAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACgAAAAA=",
        "AAAAAAAAAAAAAAAGcmVkZWVtAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFaW5kZXgAAAAAAAATAAAAAAAAAAxzaGFyZV9hbW91bnQAAAAKAAAAAA==",
        "AAAAAAAAAAAAAAAJcmViYWxhbmNlAAAAAAAAAwAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAVpbmRleAAAAAAAABMAAAAAAAAABnBhcmFtcwAAAAAH0AAAAA9SZWJhbGFuY2VQYXJhbXMAAAAAAA==",
        "AAAAAAAAAAAAAAAIcmVmYWN0b3IAAAADAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAABWluZGV4AAAAAAAAEwAAAAAAAAAGcGFyYW1zAAAAAAfQAAAADlJlZmFjdG9yUGFyYW1zAAAAAAAA",
        "AAAAAAAAAAAAAAARY2xhaW1fc3lzdGVtX2ZlZXMAAAAAAAAEAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAABWluZGV4AAAAAAAAEwAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAAAAAAtkZXN0aW5hdGlvbgAAAAATAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAASY2xhaW1fbWFuYWdlcl9mZWVzAAAAAAAEAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAABWluZGV4AAAAAAAAEwAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAAAAAAtkZXN0aW5hdGlvbgAAAAATAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAUc2V0X3ByaXZpbGVnZWRfYWRkcnMAAAAEAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADXJld2FyZHNfYWRtaW4AAAAAAAATAAAAAAAAABBvcGVyYXRpb25zX2FkbWluAAAAEwAAAAAAAAAJZmVlX2FkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAUZ2V0X3ByaXZpbGVnZWRfYWRkcnMAAAAAAAAAAQAAA+wAAAARAAAD6gAAABM=",
        "AAAAAAAAAAAAAAASZ2V0X2ZhY3RvcnlfY29uZmlnAAAAAAAAAAAAAQAAB9AAAAANRmFjdG9yeUNvbmZpZwAAAA==",
        "AAAAAAAAAAAAAAAXZ2V0X2luZGV4X2NvbnRyYWN0X3dhc20AAAAAAAAAAAEAAAPuAAAAIA==",
        "AAAAAAAAAAAAAAAUZ2V0X2RlcGxveWVkX2luZGV4ZXMAAAABAAAAAAAAAAhvcGVyYXRvcgAAABMAAAABAAAD6gAAABM=",
        "AAAAAAAAAAAAAAAYZ2V0X2FsbF9kZXBsb3llZF9pbmRleGVzAAAAAAAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAPZ2V0X2luZGV4X2NvdW50AAAAAAEAAAAAAAAACG9wZXJhdG9yAAAAEwAAAAEAAAAE",
        "AAAAAAAAAAAAAAAVZ2V0X3RvdGFsX2luZGV4X2NvdW50AAAAAAAAAAAAAAEAAAAE",
        "AAAAAAAAAAAAAAAXc2V0X2luZGV4X2NvbnRyYWN0X3dhc20AAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABNpbmRleF9jb250cmFjdF93YXNtAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAAHdmVyc2lvbgAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAANY29udHJhY3RfbmFtZQAAAAAAAAAAAAABAAAAEQ==",
        "AAAAAAAAAAAAAAAOY29tbWl0X3VwZ3JhZGUAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAANYXBwbHlfdXBncmFkZQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPuAAAAIA==",
        "AAAAAAAAAAAAAAAOcmV2ZXJ0X3VwZ3JhZGUAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAASc2V0X2VtZXJnZW5jeV9tb2RlAAAAAAACAAAAAAAAAA9lbWVyZ2VuY3lfYWRtaW4AAAAAEwAAAAAAAAAFdmFsdWUAAAAAAAABAAAAAA==",
        "AAAAAAAAAAAAAAASZ2V0X2VtZXJnZW5jeV9tb2RlAAAAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAZY29tbWl0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAAAAAALbmV3X2FkZHJlc3MAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAYYXBwbHlfdHJhbnNmZXJfb3duZXJzaGlwAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAA==",
        "AAAAAAAAAAAAAAAZcmV2ZXJ0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAA=",
        "AAAAAAAAAAAAAAASZ2V0X2Z1dHVyZV9hZGRyZXNzAAAAAAABAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAQAAABM=",
        "AAAAAQAAAAAAAAAAAAAADUZhY3RvcnlDb25maWcAAAAAAAABAAAAAAAAABNpbmRleF9jb250cmFjdF93YXNtAAAAA+4AAAAg",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAABwAAABJBY2Nlc3NDb250cm9sRXJyb3IAAAAAAAxSb2xlTm90Rm91bmQAAABlAAAAAAAAAAxVbmF1dGhvcml6ZWQAAABmAAAAAAAAAA9BZG1pbkFscmVhZHlTZXQAAAAAZwAAAAAAAAAMQmFkUm9sZVVzYWdlAAAAaAAAAAAAAAATQW5vdGhlckFjdGlvbkFjdGl2ZQAAAAtaAAAAAAAAAA5Ob0FjdGlvbkFjdGl2ZQAAAAALWwAAAAAAAAARQWN0aW9uTm90UmVhZHlZZXQAAAAAAAtc",
        "AAAAAQAAAAAAAAAAAAAAD0RleERpc3RyaWJ1dGlvbgAAAAADAAAAAAAAAAVwYXJ0cwAAAAAAAAQAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAALcHJvdG9jb2xfaWQAAAAAEA==",
        "AAAAAQAAAAAAAAAAAAAAC0luZGV4UGFyYW1zAAAAAAgAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAKY29tcG9uZW50cwAAAAAD6gAAB9AAAAAPQ29tcG9uZW50VXBkYXRlAAAAAAAAAAALZGVzY3JpcHRpb24AAAAAEAAAAAAAAAANaW5pdGlhbF9wcmljZQAAAAAAAAoAAAAAAAAACWlzX3B1YmxpYwAAAAAAAAEAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAZzeW1ib2wAAAAAABAAAAAAAAAAC3Rva2VuX3F1b3RlAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAADUluZGV4RnVuZEluZm8AAAAAAAALAAAAAAAAAAdhZGRyZXNzAAAAABMAAAAAAAAADWFkbWluX2FkZHJlc3MAAAAAAAATAAAAAAAAAA1pbml0aWFsX3ByaWNlAAAAAAAACgAAAAAAAAAJaXNfcHVibGljAAAAAAAAAQAAAAAAAAARbGFzdF9yZWJhbGFuY2VfdHMAAAAAAAAGAAAAAAAAAA9sYXN0X3VwZGF0ZWRfdHMAAAAABgAAAAAAAAATcmViYWxhbmNlX3RocmVzaG9sZAAAAAAGAAAAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAAAAAAALdG90YWxfbWludHMAAAAACgAAAAAAAAARdG90YWxfcmVkZW1wdGlvbnMAAAAAAAAKAAAAAAAAAAx0b3RhbF9zaGFyZXMAAAAK",
        "AAAAAQAAAAAAAAAAAAAAEEluZGV4RnVuZE1ldHJpY3MAAAAFAAAAAAAAAAtjdXJyZW50X25hdgAAAAAKAAAAAAAAAAtzaGFyZV9wcmljZQAAAAAKAAAAAAAAAAt0b3RhbF9taW50cwAAAAAKAAAAAAAAABF0b3RhbF9yZWRlbXB0aW9ucwAAAAAAAAoAAAAAAAAADHRvdGFsX3NoYXJlcwAAAAo=",
        "AAAAAQAAAAAAAAAAAAAAD0luZGV4RnVuZFN0YXR1cwAAAAAEAAAAAAAAAA1jYW5fcmViYWxhbmNlAAAAAAAAAQAAAAAAAAAJaXNfcHVibGljAAAAAAAAAQAAAAAAAAARbGFzdF9yZWJhbGFuY2VfdHMAAAAAAAAGAAAAAAAAABNyZWJhbGFuY2VfdGhyZXNob2xkAAAAAAY=",
        "AAAAAQAAAAAAAAAAAAAADVZvbHVtZUZlZVRpZXIAAAAAAAADAAAAAAAAAA9tYW5hZ2VyX2ZlZV9icHMAAAAABAAAAAAAAAASbWluX21vbnRobHlfdm9sdW1lAAAAAAAKAAAAAAAAABBwcm90b2NvbF9mZWVfYnBzAAAABA==",
        "AAAAAgAAAAAAAAAAAAAAC0FkYXB0ZXJUeXBlAAAAAAMAAAAAAAAAAAAAAAZOb3JtYWwAAAAAAAAAAAAAAAAACEFxdWFyaXVzAAAAAAAAAAAAAAAIU29yb3N3YXA=",
        "AAAAAQAAAAAAAAAAAAAAEkFkYXB0ZXJUcmFkZVBhcmFtcwAAAAAABgAAAAAAAAAJYW1vdW50X2luAAAAAAAACgAAAAAAAAAOYW1vdW50X291dF9taW4AAAAAAAoAAAAAAAAABWFzc2V0AAAAAAAAEQAAAAAAAAACdG8AAAAAABMAAAAAAAAACHRva2VuX2luAAAAEwAAAAAAAAAJdG9rZW5fb3V0AAAAAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAADUFkYXB0ZXJSZXN1bHQAAAAAAAADAAAAAAAAAAlhbW91bnRfaW4AAAAAAAAKAAAAAAAAAAphbW91bnRfb3V0AAAAAAAKAAAAAAAAAAdzdWNjZXNzAAAAAAE=",
        "AAAAAwAAAAAAAAAAAAAADEFkYXB0ZXJFcnJvcgAAAA4AAAAAAAAAFFByb3ZpZGVyTm90U3VwcG9ydGVkAAAAZAAAAAAAAAAVUHJvdmlkZXJOb3RDb25maWd1cmVkAAAAAAAAZQAAAAAAAAAQSW52YWxpZFRva2VuUGFpcgAAAMgAAAAAAAAADUludmFsaWRBbW91bnQAAAAAAADJAAAAAAAAAA9JbnZhbGlkU2xpcHBhZ2UAAAAAygAAAAAAAAAVSW5zdWZmaWNpZW50TGlxdWlkaXR5AAAAAAABLAAAAAAAAAAQU2xpcHBhZ2VFeGNlZWRlZAAAAS0AAAAAAAAAClN3YXBGYWlsZWQAAAAAAS4AAAAAAAAAD05vcm1hbERleEZhaWxlZAAAAAGQAAAAAAAAABJTb3Jvc3dhcFN3YXBGYWlsZWQAAAAAAZEAAAAAAAAAHVNvcm9zd2FwQWdncmVnYXRvclVuYXZhaWxhYmxlAAAAAAABkgAAAAAAAAAVSW52YWxpZFByb3ZpZGVyQ29uZmlnAAAAAAAB9AAAAAAAAAASVW5hdXRob3JpemVkQWNjZXNzAAAAAAH1AAAAAAAAABZDb250cmFjdE5vdEluaXRpYWxpemVkAAAAAAH2",
        "AAAAAQAAAAAAAAAAAAAACUNvbXBvbmVudAAAAAAAAAUAAAAAAAAAB2FkYXB0ZXIAAAAAEwAAAAAAAAAMYWRhcHRlcl90eXBlAAAH0AAAAAtBZGFwdGVyVHlwZQAAAAAAAAAABWFzc2V0AAAAAAAAEQAAAAAAAAAGb3JhY2xlAAAAAAATAAAAAAAAAAZ3ZWlnaHQAAAAAAAo=",
        "AAAAAgAAAAAAAAAAAAAAD0NvbXBvbmVudEFjdGlvbgAAAAADAAAAAAAAAAAAAAADQWRkAAAAAAAAAAAAAAAABlJlbW92ZQAAAAAAAAAAAAAAAAAMVXBkYXRlV2VpZ2h0",
        "AAAAAQAAAAAAAAAAAAAAD0NvbXBvbmVudFVwZGF0ZQAAAAAGAAAAAAAAAAZhY3Rpb24AAAAAB9AAAAAPQ29tcG9uZW50QWN0aW9uAAAAAAAAAAAHYWRhcHRlcgAAAAATAAAAAAAAAAxhZGFwdGVyX3R5cGUAAAfQAAAAC0FkYXB0ZXJUeXBlAAAAAAAAAAAKbmV3X3dlaWdodAAAAAAACgAAAAAAAAAGb3JhY2xlAAAAAAPoAAAAEwAAAAAAAAAFdG9rZW4AAAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAADlJlZmFjdG9yUGFyYW1zAAAAAAABAAAAAAAAABFjb21wb25lbnRfdXBkYXRlcwAAAAAAA+oAAAfQAAAAD0NvbXBvbmVudFVwZGF0ZQA=",
        "AAAAAQAAAAAAAAAAAAAAD1JlYmFsYW5jZVBhcmFtcwAAAAABAAAAAAAAAAp0YXJnZXRfbmF2AAAAAAPoAAAACw==",
        "AAAAAQAAAAAAAAAAAAAAE0NvbXBvbmVudEFsbG9jYXRpb24AAAAABAAAAAAAAAAJY29tcG9uZW50AAAAAAAH0AAAAAlDb21wb25lbnQAAAAAAAAAAAAAD2N1cnJlbnRfYmFsYW5jZQAAAAAKAAAAAAAAABFwZXJjZW50YWdlX29mX25hdgAAAAAAAAoAAAAAAAAADnRhcmdldF9iYWxhbmNlAAAAAAAK",
        "AAAAAQAAAAAAAAAAAAAAD1JlYmFsYW5jZVN0YXR1cwAAAAAGAAAAAAAAABZhdXRob3JpemVkX3JlYmFsYW5jZXJzAAAAAAPqAAAAEwAAAAAAAAANY2FuX3JlYmFsYW5jZQAAAAAAAAEAAAAAAAAACWlzX3B1YmxpYwAAAAAAAAEAAAAAAAAAEWxhc3RfcmViYWxhbmNlX3RzAAAAAAAABgAAAAAAAAATcmViYWxhbmNlX3RocmVzaG9sZAAAAAAGAAAAAAAAABl0aW1lX3VudGlsX25leHRfcmViYWxhbmNlAAAAAAAABg==",
        "AAAAAgAAAAAAAAAAAAAADE9yYWNsZVNvdXJjZQAAAAEAAAAAAAAAAAAAAAlSZWZsZWN0b3IAAAA=",
        "AAAABAAAAAAAAAAAAAAAC09yYWNsZUVycm9yAAAAAAUAAAAeT3JhY2xlRXJyb3I6IE9yYWNsZU5vblBvc2l0aXZlAAAAAAART3JhY2xlTm9uUG9zaXRpdmUAAAAAAAJZAAAAAAAAABFPcmFjbGVUb29Wb2xhdGlsZQAAAAAAAloAAAAAAAAAEk9yYWNsZVN0YWxlRm9yUG9vbAAAAAACWwAAAAAAAAANT3JhY2xlSW52YWxpZAAAAAAAAlwAAAAAAAAAFkZhaWxlZFRvR2V0T3JhY2xlUHJpY2UAAAAAAl0=",
        "AAAAAQAAAAAAAAAAAAAAD09yYWNsZVByaWNlRGF0YQAAAAACAAAAAAAAAAVkZWxheQAAAAAAAAYAAAAAAAAABXByaWNlAAAAAAAACg==",
        "AAAAAgAAAAAAAAAAAAAADk9yYWNsZVZhbGlkaXR5AAAAAAAFAAAAAAAAAAAAAAALTm9uUG9zaXRpdmUAAAAAAAAAAAAAAAALVG9vVm9sYXRpbGUAAAAAAAAAAAAAAAAMU3RhbGVGb3JQb29sAAAAAAAAAAAAAAAGRnJvemVuAAAAAAAAAAAAAAAAAAVWYWxpZAAAAA==",
        "AAAAAQAAAAAAAAAAAAAAFEhpc3RvcmljYWxPcmFjbGVEYXRhAAAAAwAAAAAAAAAKbGFzdF9wcmljZQAAAAAACgAAAAAAAAAPbGFzdF9wcmljZV90d2FwAAAAAAoAAAAAAAAADmxhc3RfdXBkYXRlX3RzAAAAAAAG",
        "AAAABAAAAAAAAAAAAAAADFVwZ3JhZGVFcnJvcgAAAAMAAAAMVXBncmFkZUVycm9yAAAAE0Fub3RoZXJBY3Rpb25BY3RpdmUAAAALWgAAAAAAAAAOTm9BY3Rpb25BY3RpdmUAAAAAC1sAAAAAAAAAEUFjdGlvbk5vdFJlYWR5WWV0AAAAAAALXA==",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAIAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAAAAAAAJTWF0aEVycm9yAAAAAAAB/w==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAIAAAAMU3RvcmFnZUVycm9yAAAAE1ZhbHVlTm90SW5pdGlhbGl6ZWQAAAAB9QAAAAAAAAAMVmFsdWVNaXNzaW5nAAAB9g==",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAABAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQ==",
        "AAAAAQAAAAAAAAAAAAAAE1ByaXZpbGVnZWRBZGRyZXNzZXMAAAAABQAAAAAAAAAPZW1lcmdlbmN5X2FkbWluAAAAABMAAAAAAAAAFmVtZXJnZW5jeV9wYXVzZV9hZG1pbnMAAAAAA+oAAAATAAAAAAAAABBvcGVyYXRpb25zX2FkbWluAAAAEwAAAAAAAAALcGF1c2VfYWRtaW4AAAAAEwAAAAAAAAANcmV3YXJkc19hZG1pbgAAAAAAABM=" ]),
      options
    )
  }
  public readonly fromJSON = {
    deploy_index_contract: this.txFromJSON<string>,
        mint: this.txFromJSON<null>,
        redeem: this.txFromJSON<null>,
        rebalance: this.txFromJSON<null>,
        refactor: this.txFromJSON<null>,
        claim_system_fees: this.txFromJSON<u128>,
        claim_manager_fees: this.txFromJSON<u128>,
        set_privileged_addrs: this.txFromJSON<null>,
        get_privileged_addrs: this.txFromJSON<Map<string, Array<string>>>,
        get_factory_config: this.txFromJSON<FactoryConfig>,
        get_index_contract_wasm: this.txFromJSON<Buffer>,
        get_deployed_indexes: this.txFromJSON<Array<string>>,
        get_all_deployed_indexes: this.txFromJSON<Array<string>>,
        get_index_count: this.txFromJSON<u32>,
        get_total_index_count: this.txFromJSON<u32>,
        set_index_contract_wasm: this.txFromJSON<null>,
        version: this.txFromJSON<u32>,
        contract_name: this.txFromJSON<string>,
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