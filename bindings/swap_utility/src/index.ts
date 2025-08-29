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




export const SwapError = {
  100: {message:"ProviderNotSupported"},
  101: {message:"ProviderNotConfigured"},
  200: {message:"InvalidTokenPair"},
  201: {message:"InvalidAmount"},
  202: {message:"InvalidSlippage"},
  300: {message:"InsufficientLiquidity"},
  301: {message:"SlippageExceeded"},
  302: {message:"SwapFailed"},
  400: {message:"NormalDexFailed"},
  401: {message:"SoroswapSwapFailed"},
  402: {message:"SoroswapAggregatorUnavailable"},
  500: {message:"InvalidProviderConfig"},
  501: {message:"UnauthorizedAccess"},
  502: {message:"ContractNotInitialized"}
}


export interface DexDistribution {
  parts: u32;
  path: Array<string>;
  protocol_id: string;
}

export type SwapDirection = {tag: "Buy", values: void} | {tag: "Sell", values: void};


export interface SwapParams {
  amount_in: u128;
  amount_out_min: u128;
  asset: string;
  direction: SwapDirection;
  fee_enabled: Option<boolean>;
  provider: Option<DexProvider>;
  to: string;
  token_in: string;
  token_out: string;
}

export type DexProvider = {tag: "Normal", values: void} | {tag: "Soroswap", values: void};


export interface SwapResult {
  amount_in: u128;
  amount_out: u128;
  provider_used: DexProvider;
  success: boolean;
}


export interface ProviderConfig {
  contract_address: string;
  is_active: boolean;
  max_slippage: u64;
}

export type DataKey = {tag: "ProviderConfig", values: readonly [DexProvider]} | {tag: "AdminAddress", values: void} | {tag: "Initialized", values: void} | {tag: "DefaultProvider", values: void} | {tag: "XlmTokenAddress", values: void};

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
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({admin, normal_dex_address, soroswap_address, xlm_token_address}: {admin: string, normal_dex_address: string, soroswap_address: string, xlm_token_address: string}, options?: {
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
   * Construct and simulate a is_initialized transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_initialized: (options?: {
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
   * Construct and simulate a execute_swap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  execute_swap: ({params}: {params: SwapParams}, options?: {
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
  }) => Promise<AssembledTransaction<Result<SwapResult>>>

  /**
   * Construct and simulate a execute_batch_swaps transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  execute_batch_swaps: ({swaps}: {swaps: Array<SwapParams>}, options?: {
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
  }) => Promise<AssembledTransaction<Array<Result<SwapResult>>>>

  /**
   * Construct and simulate a set_provider_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_provider_config: ({admin, provider, config}: {admin: string, provider: DexProvider, config: ProviderConfig}, options?: {
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
   * Construct and simulate a get_provider_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_provider_config: ({provider}: {provider: DexProvider}, options?: {
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
  }) => Promise<AssembledTransaction<Option<ProviderConfig>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
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
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAABAAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABJub3JtYWxfZGV4X2FkZHJlc3MAAAAAABMAAAAAAAAAEHNvcm9zd2FwX2FkZHJlc3MAAAATAAAAAAAAABF4bG1fdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAOaXNfaW5pdGlhbGl6ZWQAAAAAAAAAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAMZXhlY3V0ZV9zd2FwAAAAAQAAAAAAAAAGcGFyYW1zAAAAAAfQAAAAClN3YXBQYXJhbXMAAAAAAAEAAAPpAAAH0AAAAApTd2FwUmVzdWx0AAAAAAfQAAAACVN3YXBFcnJvcgAAAA==",
        "AAAAAAAAAAAAAAATZXhlY3V0ZV9iYXRjaF9zd2FwcwAAAAABAAAAAAAAAAVzd2FwcwAAAAAAA+oAAAfQAAAAClN3YXBQYXJhbXMAAAAAAAEAAAPqAAAD6QAAB9AAAAAKU3dhcFJlc3VsdAAAAAAH0AAAAAlTd2FwRXJyb3IAAAA=",
        "AAAAAAAAAAAAAAATc2V0X3Byb3ZpZGVyX2NvbmZpZwAAAAADAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAACHByb3ZpZGVyAAAH0AAAAAtEZXhQcm92aWRlcgAAAAAAAAAABmNvbmZpZwAAAAAH0AAAAA5Qcm92aWRlckNvbmZpZwAAAAAAAA==",
        "AAAAAAAAAAAAAAATZ2V0X3Byb3ZpZGVyX2NvbmZpZwAAAAABAAAAAAAAAAhwcm92aWRlcgAAB9AAAAALRGV4UHJvdmlkZXIAAAAAAQAAA+gAAAfQAAAADlByb3ZpZGVyQ29uZmlnAAA=",
        "AAAABAAAAAAAAAAAAAAACVN3YXBFcnJvcgAAAAAAAA4AAAAAAAAAFFByb3ZpZGVyTm90U3VwcG9ydGVkAAAAZAAAAAAAAAAVUHJvdmlkZXJOb3RDb25maWd1cmVkAAAAAAAAZQAAAAAAAAAQSW52YWxpZFRva2VuUGFpcgAAAMgAAAAAAAAADUludmFsaWRBbW91bnQAAAAAAADJAAAAAAAAAA9JbnZhbGlkU2xpcHBhZ2UAAAAAygAAAAAAAAAVSW5zdWZmaWNpZW50TGlxdWlkaXR5AAAAAAABLAAAAAAAAAAQU2xpcHBhZ2VFeGNlZWRlZAAAAS0AAAAAAAAAClN3YXBGYWlsZWQAAAAAAS4AAAAAAAAAD05vcm1hbERleEZhaWxlZAAAAAGQAAAAAAAAABJTb3Jvc3dhcFN3YXBGYWlsZWQAAAAAAZEAAAAAAAAAHVNvcm9zd2FwQWdncmVnYXRvclVuYXZhaWxhYmxlAAAAAAABkgAAAAAAAAAVSW52YWxpZFByb3ZpZGVyQ29uZmlnAAAAAAAB9AAAAAAAAAASVW5hdXRob3JpemVkQWNjZXNzAAAAAAH1AAAAAAAAABZDb250cmFjdE5vdEluaXRpYWxpemVkAAAAAAH2",
        "AAAAAQAAAAAAAAAAAAAAD0RleERpc3RyaWJ1dGlvbgAAAAADAAAAAAAAAAVwYXJ0cwAAAAAAAAQAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAALcHJvdG9jb2xfaWQAAAAAEA==",
        "AAAAAgAAAAAAAAAAAAAADVN3YXBEaXJlY3Rpb24AAAAAAAACAAAAAAAAAAAAAAADQnV5AAAAAAAAAAAAAAAABFNlbGw=",
        "AAAAAQAAAAAAAAAAAAAAClN3YXBQYXJhbXMAAAAAAAkAAAAAAAAACWFtb3VudF9pbgAAAAAAAAoAAAAAAAAADmFtb3VudF9vdXRfbWluAAAAAAAKAAAAAAAAAAVhc3NldAAAAAAAABEAAAAAAAAACWRpcmVjdGlvbgAAAAAAB9AAAAANU3dhcERpcmVjdGlvbgAAAAAAAAAAAAALZmVlX2VuYWJsZWQAAAAD6AAAAAEAAAAAAAAACHByb3ZpZGVyAAAD6AAAB9AAAAALRGV4UHJvdmlkZXIAAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAIdG9rZW5faW4AAAATAAAAAAAAAAl0b2tlbl9vdXQAAAAAAAAT",
        "AAAAAgAAAAAAAAAAAAAAC0RleFByb3ZpZGVyAAAAAAIAAAAAAAAAAAAAAAZOb3JtYWwAAAAAAAAAAAAAAAAACFNvcm9zd2Fw",
        "AAAAAQAAAAAAAAAAAAAAClN3YXBSZXN1bHQAAAAAAAQAAAAAAAAACWFtb3VudF9pbgAAAAAAAAoAAAAAAAAACmFtb3VudF9vdXQAAAAAAAoAAAAAAAAADXByb3ZpZGVyX3VzZWQAAAAAAAfQAAAAC0RleFByb3ZpZGVyAAAAAAAAAAAHc3VjY2VzcwAAAAAB",
        "AAAAAQAAAAAAAAAAAAAADlByb3ZpZGVyQ29uZmlnAAAAAAADAAAAAAAAABBjb250cmFjdF9hZGRyZXNzAAAAEwAAAAAAAAAJaXNfYWN0aXZlAAAAAAAAAQAAAAAAAAAMbWF4X3NsaXBwYWdlAAAABg==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAEAAAAAAAAADlByb3ZpZGVyQ29uZmlnAAAAAAABAAAH0AAAAAtEZXhQcm92aWRlcgAAAAAAAAAAAAAAAAxBZG1pbkFkZHJlc3MAAAAAAAAAAAAAAAtJbml0aWFsaXplZAAAAAAAAAAAAAAAAA9EZWZhdWx0UHJvdmlkZXIAAAAAAAAAAAAAAAAPWGxtVG9rZW5BZGRyZXNzAA==",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAIAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAAAAAAAJTWF0aEVycm9yAAAAAAAB/w==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAIAAAAMU3RvcmFnZUVycm9yAAAAE1ZhbHVlTm90SW5pdGlhbGl6ZWQAAAAB9QAAAAAAAAAMVmFsdWVNaXNzaW5nAAAB9g==",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAABAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQ==",
        "AAAAAQAAAAAAAAAAAAAAE1ByaXZpbGVnZWRBZGRyZXNzZXMAAAAABQAAAAAAAAAPZW1lcmdlbmN5X2FkbWluAAAAABMAAAAAAAAAFmVtZXJnZW5jeV9wYXVzZV9hZG1pbnMAAAAAA+oAAAATAAAAAAAAABBvcGVyYXRpb25zX2FkbWluAAAAEwAAAAAAAAALcGF1c2VfYWRtaW4AAAAAEwAAAAAAAAANcmV3YXJkc19hZG1pbgAAAAAAABM=",
        "AAAAAQAAAAAAAAAAAAAAC0luZGV4UGFyYW1zAAAAAA0AAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIYmFzZV9uYXYAAAAKAAAAAAAAABJibGFja2xpc3RfYWNjb3VudHMAAAAAA+oAAAATAAAAAAAAAApjb21wb25lbnRzAAAAAAPqAAAAEwAAAAAAAAALZGVzY3JpcHRpb24AAAAAEAAAAAAAAAAPaW5pdGlhbF9kZXBvc2l0AAAAAAoAAAAAAAAADWluaXRpYWxfcHJpY2UAAAAAAAAKAAAAAAAAABJtYW5hZ2VyX2ZlZV9hbW91bnQAAAAAAAoAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAZwdWJsaWMAAAAAAAEAAAAAAAAAFXJlYmFsYW5jZV9hdXRob3JpdGllcwAAAAAAA+oAAAATAAAAAAAAAAx0b2tlbl9zeW1ib2wAAAAQAAAAAAAAABJ3aGl0ZWxpc3RfYWNjb3VudHMAAAAAA+oAAAAT" ]),
      options
    )
  }
  public readonly fromJSON = {
    initialize: this.txFromJSON<null>,
        is_initialized: this.txFromJSON<boolean>,
        execute_swap: this.txFromJSON<Result<SwapResult>>,
        execute_batch_swaps: this.txFromJSON<Array<Result<SwapResult>>>,
        set_provider_config: this.txFromJSON<null>,
        get_provider_config: this.txFromJSON<Option<ProviderConfig>>
  }
}