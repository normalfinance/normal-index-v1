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
   * Construct and simulate a buy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  buy: ({params}: {params: AdapterTradeParams}, options?: {
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
   * Construct and simulate a sell transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  sell: ({params}: {params: AdapterTradeParams}, options?: {
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

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, treasury}: {admin: string, treasury: string},
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
    return ContractClient.deploy({admin, treasury}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIdHJlYXN1cnkAAAATAAAAAA==",
        "AAAAAAAAAAAAAAADYnV5AAAAAAEAAAAAAAAABnBhcmFtcwAAAAAH0AAAABJBZGFwdGVyVHJhZGVQYXJhbXMAAAAAAAEAAAAK",
        "AAAAAAAAAAAAAAAEc2VsbAAAAAEAAAAAAAAABnBhcmFtcwAAAAAH0AAAABJBZGFwdGVyVHJhZGVQYXJhbXMAAAAAAAEAAAAK",
        "AAAABAAAAAAAAAAAAAAADEFkYXB0ZXJFcnJvcgAAABMAAAAAAAAADk5vdEluaXRpYWxpemVkAAAAAAGRAAAAAAAAABJOZWdhdGl2ZU5vdEFsbG93ZWQAAAAAAZoAAAAAAAAAD0ludmFsaWRBcmd1bWVudAAAAAGbAAAAAAAAABNJbnN1ZmZpY2llbnRCYWxhbmNlAAAAAZwAAAAAAAAAEVVuZGVyZmxvd092ZXJmbG93AAAAAAABnQAAAAAAAAAPQXJpdGhtZXRpY0Vycm9yAAAAAZ4AAAAAAAAADkRpdmlzaW9uQnlaZXJvAAAAAAGfAAAAAAAAABNJbnZhbGlkU2hhcmVzTWludGVkAAAAAaAAAAAAAAAAGU9ubHlQb3NpdGl2ZUFtb3VudEFsbG93ZWQAAAAAAAGhAAAAAAAAAA1Ob3RBdXRob3JpemVkAAAAAAABogAAAAAAAAAXUHJvdG9jb2xBZGRyZXNzTm90Rm91bmQAAAABpAAAAAAAAAAPRGVhZGxpbmVFeHBpcmVkAAAAAaUAAAAAAAAADUV4dGVybmFsRXJyb3IAAAAAAAGmAAAAAAAAABFTb3Jvc3dhcFBhaXJFcnJvcgAAAAAAAacAAAAAAAAAEkFtb3VudEJlbG93TWluRHVzdAAAAAABwwAAAAAAAAAYVW5kZXJseWluZ0Ftb3VudEJlbG93TWluAAABxAAAAAAAAAAVQlRva2Vuc0Ftb3VudEJlbG93TWluAAAAAAABxQAAAAAAAAARSW50ZXJuYWxTd2FwRXJyb3IAAAAAAAHGAAAAAAAAAA5TdXBwbHlOb3RGb3VuZAAAAAABxw==",
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
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAIAAAAZTWF0aEVycm9yOiBOdW1iZXJPdmVyZmxvdwAAAAAAAA5OdW1iZXJPdmVyZmxvdwAAAAAB/gAAAAAAAAAJTWF0aEVycm9yAAAAAAAB/w==",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAIAAAAMU3RvcmFnZUVycm9yAAAAE1ZhbHVlTm90SW5pdGlhbGl6ZWQAAAAB9QAAAAAAAAAMVmFsdWVNaXNzaW5nAAAB9g==",
        "AAAABAAAAAAAAAAAAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAABAAAAD1ZhbGlkYXRpb25FcnJvcgAAAAAMSW52YWxpZFRva2VuAAADIQ==",
        "AAAAAQAAAAAAAAAAAAAAE1ByaXZpbGVnZWRBZGRyZXNzZXMAAAAABQAAAAAAAAAPZW1lcmdlbmN5X2FkbWluAAAAABMAAAAAAAAAFmVtZXJnZW5jeV9wYXVzZV9hZG1pbnMAAAAAA+oAAAATAAAAAAAAABBvcGVyYXRpb25zX2FkbWluAAAAEwAAAAAAAAALcGF1c2VfYWRtaW4AAAAAEwAAAAAAAAANcmV3YXJkc19hZG1pbgAAAAAAABM=" ]),
      options
    )
  }
  public readonly fromJSON = {
    buy: this.txFromJSON<u128>,
        sell: this.txFromJSON<u128>
  }
}