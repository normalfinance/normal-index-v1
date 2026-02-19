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




export const AdapterRegistryError = {
  1: {message:"Unauthorized"},
  2: {message:"AdapterNameNotFound"},
  3: {message:"AdapterAddressNotFound"},
  4: {message:"AdapterAddressAlreadyAssigned"}
}

export type DataKey = {tag: "Admin", values: void} | {tag: "AdapterByName", values: readonly [string]} | {tag: "NameByAdapter", values: readonly [string]} | {tag: "AdapterNames", values: void};

export interface Client {
  /**
   * Construct and simulate a set_adapter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_adapter: ({admin, name, adapter}: {admin: string, name: string, adapter: string}, options?: {
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
   * Construct and simulate a get_adapter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_adapter: ({name}: {name: string}, options?: {
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
   * Construct and simulate a get_adapter_safe transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_adapter_safe: ({name}: {name: string}, options?: {
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
  }) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a get_adapter_name transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_adapter_name: ({adapter}: {adapter: string}, options?: {
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
   * Construct and simulate a get_adapters transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_adapters: (options?: {
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
  }) => Promise<AssembledTransaction<Map<string, string>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin}: {admin: string},
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
    return ContractClient.deploy({admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAALc2V0X2FkYXB0ZXIAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAARuYW1lAAAAEQAAAAAAAAAHYWRhcHRlcgAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAALZ2V0X2FkYXB0ZXIAAAAAAQAAAAAAAAAEbmFtZQAAABEAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAQZ2V0X2FkYXB0ZXJfc2FmZQAAAAEAAAAAAAAABG5hbWUAAAARAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAAQZ2V0X2FkYXB0ZXJfbmFtZQAAAAEAAAAAAAAAB2FkYXB0ZXIAAAAAEwAAAAEAAAAR",
        "AAAAAAAAAAAAAAAMZ2V0X2FkYXB0ZXJzAAAAAAAAAAEAAAPsAAAAEQAAABM=",
        "AAAABAAAAAAAAAAAAAAAFEFkYXB0ZXJSZWdpc3RyeUVycm9yAAAABAAAAAAAAAAMVW5hdXRob3JpemVkAAAAAQAAAAAAAAATQWRhcHRlck5hbWVOb3RGb3VuZAAAAAACAAAAAAAAABZBZGFwdGVyQWRkcmVzc05vdEZvdW5kAAAAAAADAAAAAAAAAB1BZGFwdGVyQWRkcmVzc0FscmVhZHlBc3NpZ25lZAAAAAAAAAQ=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABAAAAAAAAAAAAAAABUFkbWluAAAAAAAAAQAAAAAAAAANQWRhcHRlckJ5TmFtZQAAAAAAAAEAAAARAAAAAQAAAAAAAAANTmFtZUJ5QWRhcHRlcgAAAAAAAAEAAAATAAAAAAAAAAAAAAAMQWRhcHRlck5hbWVz" ]),
      options
    )
  }
  public readonly fromJSON = {
    set_adapter: this.txFromJSON<null>,
        get_adapter: this.txFromJSON<string>,
        get_adapter_safe: this.txFromJSON<Option<string>>,
        get_adapter_name: this.txFromJSON<string>,
        get_adapters: this.txFromJSON<Map<string, string>>
  }
}