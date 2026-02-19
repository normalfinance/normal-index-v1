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




export const AdapterError = {
  401: {message:"NotInitialized"},
  100: {message:"MissingPoolHashes"},
  101: {message:"WrongMinimumPathLength"},
  102: {message:"WrongPoolHashesLength"},
  410: {message:"NegativeNotAllowed"},
  411: {message:"InvalidArgument"},
  412: {message:"InsufficientBalance"},
  413: {message:"UnderflowOverflow"},
  414: {message:"ArithmeticError"},
  415: {message:"DivisionByZero"},
  416: {message:"InvalidSharesMinted"},
  417: {message:"OnlyPositiveAmountAllowed"},
  418: {message:"NotAuthorized"},
  420: {message:"ProtocolAddressNotFound"},
  421: {message:"DeadlineExpired"},
  422: {message:"ExternalError"},
  423: {message:"SoroswapPairError"},
  451: {message:"AmountBelowMinDust"},
  452: {message:"UnderlyingAmountBelowMin"},
  453: {message:"BTokensAmountBelowMin"},
  454: {message:"InternalSwapError"},
  455: {message:"SupplyNotFound"}
}


export interface AdapterMetadata {
  address: Option<Map<string, string>>;
  number: Option<Map<string, i128>>;
}


export interface AdapterTradeParams {
  amount_in: u128;
  amount_out_min: u128;
  asset: string;
  metadata: Option<AdapterMetadata>;
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
  asset: string;
  oracle: string;
  weight: u128;
}

export type ComponentAction = {tag: "Add", values: void} | {tag: "Remove", values: void} | {tag: "UpdateWeight", values: void} | {tag: "UpdateOracle", values: void} | {tag: "UpdateAdapter", values: void};


export interface ComponentUpdate {
  action: ComponentAction;
  new_adapter: Option<string>;
  new_oracle: Option<string>;
  new_weight: Option<u128>;
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
  can_rebalance: boolean;
  is_public: boolean;
  last_rebalance_ts: u64;
  rebalance_authorities: Array<string>;
  rebalance_threshold: u64;
  time_until_next_rebalance: u64;
}


export interface DexDistribution {
  parts: u32;
  path: Array<string>;
  protocol_id: string;
}


export interface IndexFundAuthorities {
  admin: string;
  emergency_admin: string;
  fee_admin: string;
  operations_admin: string;
  rebalance_authorities: Array<string>;
  rewards_admin: string;
}


/**
 * Parameters used when creating a new index
 */
export interface DeployIndexParams {
  /**
 * The addresses which administrate the index
 */
authorities: IndexFundAuthorities;
  /**
 * The assets within the index
 */
components: Array<ComponentUpdate>;
  /**
 * The index description (Equally tracks the top 5 cryptocurrencies)
 */
description: string;
  /**
 * The starting share price of the index
 */
initial_price: u128;
  /**
 * The index visibility (public or private)
 */
is_public: boolean;
  /**
 * The index name (Normal Top 5 Crypto Index)
 */
name: string;
  /**
 * The address of the token used to mint the index (usually USDC)
 */
quote_token: string;
  /**
 * The index token symbol (NTOP5)
 */
symbol: string;
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


export interface VolumeFeeTier {
  manager_fee_bps: u32;
  min_monthly_volume: u128;
  protocol_fee_bps: u32;
}

export const MathError = {
  /**
   * MathError: NumberOverflow
   */
  510: {message:"NumberOverflow"},
  /**
   * MathError: Generic math error
   */
  511: {message:"MathError"},
  /**
   * MathError: Addition operation caused overflow
   */
  512: {message:"AdditionOverflow"},
  /**
   * MathError: Subtraction operation caused underflow
   */
  513: {message:"SubtractionUnderflow"},
  /**
   * MathError: Multiplication operation caused overflow
   */
  514: {message:"MultiplicationOverflow"},
  /**
   * MathError: Division by zero
   */
  515: {message:"DivisionByZero"},
  /**
   * MathError: Type conversion overflow
   */
  516: {message:"ConversionOverflow"},
  /**
   * MathError: Attempted to convert negative value to unsigned type
   */
  517: {message:"NegativeToUnsigned"},
  /**
   * MathError: Fixed-point arithmetic overflow
   */
  518: {message:"FixedPointOverflow"}
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

export interface Client {
  /**
   * Construct and simulate a swap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap: ({params}: {params: AdapterTradeParams}, options?: {
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
  }) => Promise<AssembledTransaction<Result<u128>>>

  /**
   * Construct and simulate a get_protocol_id transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_protocol_id: (options?: {
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
  }) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_protocol_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_protocol_address: (options?: {
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
  }) => Promise<AssembledTransaction<Result<string>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, protocol_id, protocol_address}: {admin: string, protocol_id: string, protocol_address: string},
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
    return ContractClient.deploy({admin, protocol_id, protocol_address}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAALcHJvdG9jb2xfaWQAAAAAEAAAAAAAAAAQcHJvdG9jb2xfYWRkcmVzcwAAABMAAAAA",
        "AAAAAAAAAAAAAAAEc3dhcAAAAAEAAAAAAAAABnBhcmFtcwAAAAAH0AAAABJBZGFwdGVyVHJhZGVQYXJhbXMAAAAAAAEAAAPpAAAACgAAB9AAAAAMQWRhcHRlckVycm9y",
        "AAAAAAAAAAAAAAAPZ2V0X3Byb3RvY29sX2lkAAAAAAAAAAABAAAD6QAAABAAAAfQAAAADEFkYXB0ZXJFcnJvcg==",
        "AAAAAAAAAAAAAAAUZ2V0X3Byb3RvY29sX2FkZHJlc3MAAAAAAAAAAQAAA+kAAAATAAAH0AAAAAxBZGFwdGVyRXJyb3I=",
        "AAAABAAAAAAAAAAAAAAADEFkYXB0ZXJFcnJvcgAAABYAAAAAAAAADk5vdEluaXRpYWxpemVkAAAAAAGRAAAAAAAAABFNaXNzaW5nUG9vbEhhc2hlcwAAAAAAAGQAAAAAAAAAFldyb25nTWluaW11bVBhdGhMZW5ndGgAAAAAAGUAAAAAAAAAFVdyb25nUG9vbEhhc2hlc0xlbmd0aAAAAAAAAGYAAAAAAAAAEk5lZ2F0aXZlTm90QWxsb3dlZAAAAAABmgAAAAAAAAAPSW52YWxpZEFyZ3VtZW50AAAAAZsAAAAAAAAAE0luc3VmZmljaWVudEJhbGFuY2UAAAABnAAAAAAAAAARVW5kZXJmbG93T3ZlcmZsb3cAAAAAAAGdAAAAAAAAAA9Bcml0aG1ldGljRXJyb3IAAAABngAAAAAAAAAORGl2aXNpb25CeVplcm8AAAAAAZ8AAAAAAAAAE0ludmFsaWRTaGFyZXNNaW50ZWQAAAABoAAAAAAAAAAZT25seVBvc2l0aXZlQW1vdW50QWxsb3dlZAAAAAAAAaEAAAAAAAAADU5vdEF1dGhvcml6ZWQAAAAAAAGiAAAAAAAAABdQcm90b2NvbEFkZHJlc3NOb3RGb3VuZAAAAAGkAAAAAAAAAA9EZWFkbGluZUV4cGlyZWQAAAABpQAAAAAAAAANRXh0ZXJuYWxFcnJvcgAAAAAAAaYAAAAAAAAAEVNvcm9zd2FwUGFpckVycm9yAAAAAAABpwAAAAAAAAASQW1vdW50QmVsb3dNaW5EdXN0AAAAAAHDAAAAAAAAABhVbmRlcmx5aW5nQW1vdW50QmVsb3dNaW4AAAHEAAAAAAAAABVCVG9rZW5zQW1vdW50QmVsb3dNaW4AAAAAAAHFAAAAAAAAABFJbnRlcm5hbFN3YXBFcnJvcgAAAAAAAcYAAAAAAAAADlN1cHBseU5vdEZvdW5kAAAAAAHH",
        "AAAAAQAAAAAAAAAAAAAAD0FkYXB0ZXJNZXRhZGF0YQAAAAACAAAAAAAAAAdhZGRyZXNzAAAAA+gAAAPsAAAAEQAAABMAAAAAAAAABm51bWJlcgAAAAAD6AAAA+wAAAARAAAACw==",
        "AAAAAQAAAAAAAAAAAAAAEkFkYXB0ZXJUcmFkZVBhcmFtcwAAAAAABwAAAAAAAAAJYW1vdW50X2luAAAAAAAACgAAAAAAAAAOYW1vdW50X291dF9taW4AAAAAAAoAAAAAAAAABWFzc2V0AAAAAAAAEQAAAAAAAAAIbWV0YWRhdGEAAAPoAAAH0AAAAA9BZGFwdGVyTWV0YWRhdGEAAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAIdG9rZW5faW4AAAATAAAAAAAAAAl0b2tlbl9vdXQAAAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAADUFkYXB0ZXJSZXN1bHQAAAAAAAADAAAAAAAAAAlhbW91bnRfaW4AAAAAAAAKAAAAAAAAAAphbW91bnRfb3V0AAAAAAAKAAAAAAAAAAdzdWNjZXNzAAAAAAE=",
        "AAAAAwAAAAAAAAAAAAAADEFkYXB0ZXJFcnJvcgAAAA4AAAAAAAAAFFByb3ZpZGVyTm90U3VwcG9ydGVkAAAAZAAAAAAAAAAVUHJvdmlkZXJOb3RDb25maWd1cmVkAAAAAAAAZQAAAAAAAAAQSW52YWxpZFRva2VuUGFpcgAAAMgAAAAAAAAADUludmFsaWRBbW91bnQAAAAAAADJAAAAAAAAAA9JbnZhbGlkU2xpcHBhZ2UAAAAAygAAAAAAAAAVSW5zdWZmaWNpZW50TGlxdWlkaXR5AAAAAAABLAAAAAAAAAAQU2xpcHBhZ2VFeGNlZWRlZAAAAS0AAAAAAAAAClN3YXBGYWlsZWQAAAAAAS4AAAAAAAAAD05vcm1hbERleEZhaWxlZAAAAAGQAAAAAAAAABJTb3Jvc3dhcFN3YXBGYWlsZWQAAAAAAZEAAAAAAAAAHVNvcm9zd2FwQWdncmVnYXRvclVuYXZhaWxhYmxlAAAAAAABkgAAAAAAAAAVSW52YWxpZFByb3ZpZGVyQ29uZmlnAAAAAAAB9AAAAAAAAAASVW5hdXRob3JpemVkQWNjZXNzAAAAAAH1AAAAAAAAABZDb250cmFjdE5vdEluaXRpYWxpemVkAAAAAAH2",
        "AAAAAQAAAAAAAAAAAAAACUNvbXBvbmVudAAAAAAAAAQAAAAAAAAAB2FkYXB0ZXIAAAAAEQAAAAAAAAAFYXNzZXQAAAAAAAARAAAAAAAAAAZvcmFjbGUAAAAAABMAAAAAAAAABndlaWdodAAAAAAACg==",
        "AAAAAgAAAAAAAAAAAAAAD0NvbXBvbmVudEFjdGlvbgAAAAAFAAAAAAAAAAAAAAADQWRkAAAAAAAAAAAAAAAABlJlbW92ZQAAAAAAAAAAAAAAAAAMVXBkYXRlV2VpZ2h0AAAAAAAAAAAAAAAMVXBkYXRlT3JhY2xlAAAAAAAAAAAAAAANVXBkYXRlQWRhcHRlcgAAAA==",
        "AAAAAQAAAAAAAAAAAAAAD0NvbXBvbmVudFVwZGF0ZQAAAAAFAAAAAAAAAAZhY3Rpb24AAAAAB9AAAAAPQ29tcG9uZW50QWN0aW9uAAAAAAAAAAALbmV3X2FkYXB0ZXIAAAAD6AAAABEAAAAAAAAACm5ld19vcmFjbGUAAAAAA+gAAAATAAAAAAAAAApuZXdfd2VpZ2h0AAAAAAPoAAAACgAAAAAAAAAFdG9rZW4AAAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAADlJlZmFjdG9yUGFyYW1zAAAAAAABAAAAAAAAABFjb21wb25lbnRfdXBkYXRlcwAAAAAAA+oAAAfQAAAAD0NvbXBvbmVudFVwZGF0ZQA=",
        "AAAAAQAAAAAAAAAAAAAAD1JlYmFsYW5jZVBhcmFtcwAAAAABAAAAAAAAAAp0YXJnZXRfbmF2AAAAAAPoAAAACw==",
        "AAAAAQAAAAAAAAAAAAAAE0NvbXBvbmVudEFsbG9jYXRpb24AAAAABAAAAAAAAAAJY29tcG9uZW50AAAAAAAH0AAAAAlDb21wb25lbnQAAAAAAAAAAAAAD2N1cnJlbnRfYmFsYW5jZQAAAAAKAAAAAAAAABFwZXJjZW50YWdlX29mX25hdgAAAAAAAAoAAAAAAAAADnRhcmdldF9iYWxhbmNlAAAAAAAK",
        "AAAAAQAAAAAAAAAAAAAAD1JlYmFsYW5jZVN0YXR1cwAAAAAGAAAAAAAAAA1jYW5fcmViYWxhbmNlAAAAAAAAAQAAAAAAAAAJaXNfcHVibGljAAAAAAAAAQAAAAAAAAARbGFzdF9yZWJhbGFuY2VfdHMAAAAAAAAGAAAAAAAAABVyZWJhbGFuY2VfYXV0aG9yaXRpZXMAAAAAAAPqAAAAEwAAAAAAAAATcmViYWxhbmNlX3RocmVzaG9sZAAAAAAGAAAAAAAAABl0aW1lX3VudGlsX25leHRfcmViYWxhbmNlAAAAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAAD0RleERpc3RyaWJ1dGlvbgAAAAADAAAAAAAAAAVwYXJ0cwAAAAAAAAQAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAALcHJvdG9jb2xfaWQAAAAAEA==",
        "AAAAAQAAAAAAAAAAAAAAFEluZGV4RnVuZEF1dGhvcml0aWVzAAAABgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAA9lbWVyZ2VuY3lfYWRtaW4AAAAAEwAAAAAAAAAJZmVlX2FkbWluAAAAAAAAEwAAAAAAAAAQb3BlcmF0aW9uc19hZG1pbgAAABMAAAAAAAAAFXJlYmFsYW5jZV9hdXRob3JpdGllcwAAAAAAA+oAAAATAAAAAAAAAA1yZXdhcmRzX2FkbWluAAAAAAAAEw==",
        "AAAAAQAAAClQYXJhbWV0ZXJzIHVzZWQgd2hlbiBjcmVhdGluZyBhIG5ldyBpbmRleAAAAAAAAAAAAAARRGVwbG95SW5kZXhQYXJhbXMAAAAAAAAIAAAAKlRoZSBhZGRyZXNzZXMgd2hpY2ggYWRtaW5pc3RyYXRlIHRoZSBpbmRleAAAAAAAC2F1dGhvcml0aWVzAAAAB9AAAAAUSW5kZXhGdW5kQXV0aG9yaXRpZXMAAAAbVGhlIGFzc2V0cyB3aXRoaW4gdGhlIGluZGV4AAAAAApjb21wb25lbnRzAAAAAAPqAAAH0AAAAA9Db21wb25lbnRVcGRhdGUAAAAAQVRoZSBpbmRleCBkZXNjcmlwdGlvbiAoRXF1YWxseSB0cmFja3MgdGhlIHRvcCA1IGNyeXB0b2N1cnJlbmNpZXMpAAAAAAAAC2Rlc2NyaXB0aW9uAAAAABAAAAAlVGhlIHN0YXJ0aW5nIHNoYXJlIHByaWNlIG9mIHRoZSBpbmRleAAAAAAAAA1pbml0aWFsX3ByaWNlAAAAAAAACgAAAChUaGUgaW5kZXggdmlzaWJpbGl0eSAocHVibGljIG9yIHByaXZhdGUpAAAACWlzX3B1YmxpYwAAAAAAAAEAAAAqVGhlIGluZGV4IG5hbWUgKE5vcm1hbCBUb3AgNSBDcnlwdG8gSW5kZXgpAAAAAAAEbmFtZQAAABAAAAA+VGhlIGFkZHJlc3Mgb2YgdGhlIHRva2VuIHVzZWQgdG8gbWludCB0aGUgaW5kZXggKHVzdWFsbHkgVVNEQykAAAAAAAtxdW90ZV90b2tlbgAAAAATAAAAHlRoZSBpbmRleCB0b2tlbiBzeW1ib2wgKE5UT1A1KQAAAAAABnN5bWJvbAAAAAAAEA==",
        "AAAAAQAAAAAAAAAAAAAADUluZGV4RnVuZEluZm8AAAAAAAALAAAAAAAAAAdhZGRyZXNzAAAAABMAAAAAAAAADWFkbWluX2FkZHJlc3MAAAAAAAATAAAAAAAAAA1pbml0aWFsX3ByaWNlAAAAAAAACgAAAAAAAAAJaXNfcHVibGljAAAAAAAAAQAAAAAAAAARbGFzdF9yZWJhbGFuY2VfdHMAAAAAAAAGAAAAAAAAAA9sYXN0X3VwZGF0ZWRfdHMAAAAABgAAAAAAAAATcmViYWxhbmNlX3RocmVzaG9sZAAAAAAGAAAAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAAAAAAALdG90YWxfbWludHMAAAAACgAAAAAAAAARdG90YWxfcmVkZW1wdGlvbnMAAAAAAAAKAAAAAAAAAAx0b3RhbF9zaGFyZXMAAAAK",
        "AAAAAQAAAAAAAAAAAAAAEEluZGV4RnVuZE1ldHJpY3MAAAAFAAAAAAAAAAtjdXJyZW50X25hdgAAAAAKAAAAAAAAAAtzaGFyZV9wcmljZQAAAAAKAAAAAAAAAAt0b3RhbF9taW50cwAAAAAKAAAAAAAAABF0b3RhbF9yZWRlbXB0aW9ucwAAAAAAAAoAAAAAAAAADHRvdGFsX3NoYXJlcwAAAAo=",
        "AAAAAQAAAAAAAAAAAAAAD0luZGV4RnVuZFN0YXR1cwAAAAAEAAAAAAAAAA1jYW5fcmViYWxhbmNlAAAAAAAAAQAAAAAAAAAJaXNfcHVibGljAAAAAAAAAQAAAAAAAAARbGFzdF9yZWJhbGFuY2VfdHMAAAAAAAAGAAAAAAAAABNyZWJhbGFuY2VfdGhyZXNob2xkAAAAAAY=",
        "AAAAAgAAAAAAAAAAAAAADE9yYWNsZVNvdXJjZQAAAAEAAAAAAAAAAAAAAAlSZWZsZWN0b3IAAAA=",
        "AAAABAAAAAAAAAAAAAAAC09yYWNsZUVycm9yAAAAAAUAAAAeT3JhY2xlRXJyb3I6IE9yYWNsZU5vblBvc2l0aXZlAAAAAAART3JhY2xlTm9uUG9zaXRpdmUAAAAAAAJZAAAAAAAAABFPcmFjbGVUb29Wb2xhdGlsZQAAAAAAAloAAAAAAAAAEk9yYWNsZVN0YWxlRm9yUG9vbAAAAAACWwAAAAAAAAANT3JhY2xlSW52YWxpZAAAAAAAAlwAAAAAAAAAFkZhaWxlZFRvR2V0T3JhY2xlUHJpY2UAAAAAAl0=",
        "AAAAAQAAAAAAAAAAAAAAD09yYWNsZVByaWNlRGF0YQAAAAACAAAAAAAAAAVkZWxheQAAAAAAAAYAAAAAAAAABXByaWNlAAAAAAAACg==",
        "AAAAAgAAAAAAAAAAAAAADk9yYWNsZVZhbGlkaXR5AAAAAAAFAAAAAAAAAAAAAAALTm9uUG9zaXRpdmUAAAAAAAAAAAAAAAALVG9vVm9sYXRpbGUAAAAAAAAAAAAAAAAMU3RhbGVGb3JQb29sAAAAAAAAAAAAAAAGRnJvemVuAAAAAAAAAAAAAAAAAAVWYWxpZAAAAA==",
        "AAAAAQAAAAAAAAAAAAAAFEhpc3RvcmljYWxPcmFjbGVEYXRhAAAAAwAAAAAAAAAKbGFzdF9wcmljZQAAAAAACgAAAAAAAAAPbGFzdF9wcmljZV90d2FwAAAAAAoAAAAAAAAADmxhc3RfdXBkYXRlX3RzAAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAADVZvbHVtZUZlZVRpZXIAAAAAAAADAAAAAAAAAA9tYW5hZ2VyX2ZlZV9icHMAAAAABAAAAAAAAAASbWluX21vbnRobHlfdm9sdW1lAAAAAAAKAAAAAAAAABBwcm90b2NvbF9mZWVfYnBzAAAABA==",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAkAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAB1NYXRoRXJyb3I6IEdlbmVyaWMgbWF0aCBlcnJvcgAAAAAAAAlNYXRoRXJyb3IAAAAAAAH/AAAALU1hdGhFcnJvcjogQWRkaXRpb24gb3BlcmF0aW9uIGNhdXNlZCBvdmVyZmxvdwAAAAAAABBBZGRpdGlvbk92ZXJmbG93AAACAAAAADFNYXRoRXJyb3I6IFN1YnRyYWN0aW9uIG9wZXJhdGlvbiBjYXVzZWQgdW5kZXJmbG93AAAAAAAAFFN1YnRyYWN0aW9uVW5kZXJmbG93AAACAQAAADNNYXRoRXJyb3I6IE11bHRpcGxpY2F0aW9uIG9wZXJhdGlvbiBjYXVzZWQgb3ZlcmZsb3cAAAAAFk11bHRpcGxpY2F0aW9uT3ZlcmZsb3cAAAAAAgIAAAAbTWF0aEVycm9yOiBEaXZpc2lvbiBieSB6ZXJvAAAAAA5EaXZpc2lvbkJ5WmVybwAAAAACAwAAACNNYXRoRXJyb3I6IFR5cGUgY29udmVyc2lvbiBvdmVyZmxvdwAAAAASQ29udmVyc2lvbk92ZXJmbG93AAAAAAIEAAAAP01hdGhFcnJvcjogQXR0ZW1wdGVkIHRvIGNvbnZlcnQgbmVnYXRpdmUgdmFsdWUgdG8gdW5zaWduZWQgdHlwZQAAAAASTmVnYXRpdmVUb1Vuc2lnbmVkAAAAAAIFAAAAKk1hdGhFcnJvcjogRml4ZWQtcG9pbnQgYXJpdGhtZXRpYyBvdmVyZmxvdwAAAAAAEkZpeGVkUG9pbnRPdmVyZmxvdwAAAAACBg==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAIAAAAMU3RvcmFnZUVycm9yAAAAE1ZhbHVlTm90SW5pdGlhbGl6ZWQAAAAB9QAAAAAAAAAMVmFsdWVNaXNzaW5nAAAB9g==",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAABAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQ==" ]),
      options
    )
  }
  public readonly fromJSON = {
    swap: this.txFromJSON<Result<u128>>,
        get_protocol_id: this.txFromJSON<Result<string>>,
        get_protocol_address: this.txFromJSON<Result<string>>
  }
}