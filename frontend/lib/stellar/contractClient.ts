/* eslint-disable @typescript-eslint/no-explicit-any, @typescript-eslint/no-unused-vars */
import { Contract, xdr, rpc, TransactionBuilder, Networks } from "@stellar/stellar-sdk";
import type { StellarNetwork } from "./networks";
import { CONTRACT_CONFIG } from "@/lib/contract/config";
import { trackContractInteraction, trackError } from "@/lib/analytics";

export type ContractClientErrorCode =
  | "CONTRACT_NOT_CONFIGURED"
  | "RPC_UNAVAILABLE"
  | "NETWORK_ERROR"
  | "SIMULATION_FAILED"
  | "UNKNOWN";

export class ContractClientError extends Error {
  code: ContractClientErrorCode;
  recoverable: boolean;
  userMessage: string;

  constructor(params: {
    code: ContractClientErrorCode;
    message: string;
    userMessage: string;
    recoverable: boolean;
    cause?: unknown;
  }) {
    super(params.message);
    this.name = "ContractClientError";
    this.code = params.code;
    this.recoverable = params.recoverable;
    this.userMessage = params.userMessage;
    if (params.cause !== undefined) {
      (this as any).cause = params.cause;
    }
  }
}

function isContractClientError(error: unknown): error is ContractClientError {
  return Boolean(error) && typeof error === "object" && (error as any).name === "ContractClientError";
}

function normalizeContractClientError(error: unknown): ContractClientError {
  if (isContractClientError(error)) return error;

  const message = error instanceof Error ? error.message : String(error);
  const lower = message.toLowerCase();

  if (lower.includes("contract id not configured") || lower.includes("next_public_contract_id")) {
    return new ContractClientError({
      code: "CONTRACT_NOT_CONFIGURED",
      message,
      userMessage: "Contract is not configured. Add NEXT_PUBLIC_CONTRACT_ID and reload.",
      recoverable: true,
      cause: error,
    });
  }

  if (
    lower.includes("failed to fetch") ||
    lower.includes("network") ||
    lower.includes("timeout") ||
    lower.includes("econn") ||
    lower.includes("502") ||
    lower.includes("503") ||
    lower.includes("504")
  ) {
    return new ContractClientError({
      code: "RPC_UNAVAILABLE",
      message,
      userMessage: "The contract RPC appears unavailable. Try again in a moment.",
      recoverable: true,
      cause: error,
    });
  }

  return new ContractClientError({
    code: "UNKNOWN",
    message,
    userMessage: "Something went wrong while reading contract data.",
    recoverable: false,
    cause: error,
  });
}

export async function retryAsync<T>(
  fn: () => Promise<T>,
  options?: {
    retries?: number;
    baseDelayMs?: number;
    shouldRetry?: (err: unknown) => boolean;
  }
): Promise<T> {
  const retries = Math.max(0, options?.retries ?? 2);
  const baseDelayMs = Math.max(0, options?.baseDelayMs ?? 350);
  const shouldRetry =
    options?.shouldRetry ??
    ((err: unknown) => {
      const normalized = normalizeContractClientError(err);
      return normalized.recoverable && (normalized.code === "RPC_UNAVAILABLE" || normalized.code === "NETWORK_ERROR");
    });

  let lastError: unknown;
  for (let attempt = 0; attempt <= retries; attempt += 1) {
    try {
      return await fn();
    } catch (err) {
      lastError = err;
      if (attempt === retries || !shouldRetry(err)) {
        throw err;
      }
      await new Promise((resolve) => setTimeout(resolve, baseDelayMs * Math.pow(2, attempt)));
    }
  }

  throw lastError;
}

function scValToJs(scVal: xdr.ScVal): any {
  switch (scVal.switch()) {
    case xdr.ScValType.scvBool():
      return scVal.b();
    case xdr.ScValType.scvVoid():
      return null;
    case xdr.ScValType.scvError():
      return null;
    case xdr.ScValType.scvU32():
      return scVal.u32();
    case xdr.ScValType.scvI32():
      return scVal.i32();
    case xdr.ScValType.scvU64():
      return Number(scVal.u64());
    case xdr.ScValType.scvI64():
      return Number(scVal.i64());
    case xdr.ScValType.scvTimepoint():
      return Number(scVal.timepoint());
    case xdr.ScValType.scvDuration():
      return Number(scVal.duration());
    case xdr.ScValType.scvU128():
      const u128 = scVal.u128();
      return Number(u128.lo());
    case xdr.ScValType.scvI128():
      const i128 = scVal.i128();
      return Number(i128.lo());
    case xdr.ScValType.scvU256():
      return 0;
    case xdr.ScValType.scvI256():
      return 0;
    case xdr.ScValType.scvBytes():
      return Buffer.from(scVal.bytes()).toString("base64");
    case xdr.ScValType.scvString():
      return scVal.str().toString();
    case xdr.ScValType.scvSymbol():
      return scVal.sym().toString();
    case xdr.ScValType.scvVec():
      const vec = scVal.vec();
      if (!vec) return [];
      return Array.from(vec).map((v) => scValToJs(v));
    case xdr.ScValType.scvMap():
      const map = scVal.map();
      if (!map) return {};
      const obj: Record<string, any> = {};
      for (const entry of map) {
        const key = scValToJs(entry.key());
        const val = scValToJs(entry.val());
        obj[String(key)] = val;
      }
      return obj;
    case xdr.ScValType.scvAddress():
      return scVal.address().toString();
    case xdr.ScValType.scvContractInstance():
      return scVal.instance().toString();
    default:
      return null;
  }
}

function scValToString(scVal: xdr.ScVal): string {
  if (scVal.switch() === xdr.ScValType.scvString()) {
    return scVal.str().toString();
  }
  if (scVal.switch() === xdr.ScValType.scvSymbol()) {
    return scVal.sym().toString();
  }
  if (scVal.switch() === xdr.ScValType.scvAddress()) {
    return scVal.address().toString();
  }
  const js = scValToJs(scVal);
  return js ? String(js) : "";
}

export type ContractClientConfig = {
  contractId: string;
  network: StellarNetwork;
  rpcUrl?: string;
};

function getRpcUrl(network: StellarNetwork, customUrl?: string): string {
  if (customUrl) return customUrl;
  
  switch (network) {
    case "testnet":
      return "https://soroban-testnet.stellar.org";
    case "mainnet":
      return "https://soroban-rpc.mainnet.stellar.org";
    case "futurenet":
      return "https://rpc-futurenet.stellar.org";
    default:
      return "https://soroban-testnet.stellar.org";
  }
}

export function createContractClient(config?: Partial<ContractClientConfig>) {
  const contractId = config?.contractId || CONTRACT_CONFIG.CONTRACT_ID;
  const network = config?.network || CONTRACT_CONFIG.NETWORK;
  const rpcUrl = config?.rpcUrl || CONTRACT_CONFIG.RPC_URL;
  
  if (!contractId) {
    throw new ContractClientError({
      code: "CONTRACT_NOT_CONFIGURED",
      message: "Contract ID not configured",
      userMessage: "Contract is not configured. Add NEXT_PUBLIC_CONTRACT_ID and reload.",
      recoverable: true,
    });
  }

  const rpcServer = new rpc.Server(rpcUrl, { allowHttp: true });
  const contract = new Contract(contractId);

  const now = () => Date.now();

  return {
    async ping(): Promise<string> {
      return "ok";
    },

    async get_product_event_ids(productId: string): Promise<number[]> {
      const startedAt = now();
      try {
        const operation = contract.call("get_product_event_ids", xdr.ScVal.scvString(productId));
        const networkPassphrase = network === "testnet" ? Networks.TESTNET : network === "mainnet" ? Networks.PUBLIC : Networks.TESTNET;
        const dummyAccount = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
        let sourceAccount: any;
        try {
          sourceAccount = await rpcServer.getAccount(dummyAccount);
        } catch {
          sourceAccount = {
            accountId: dummyAccount,
            sequenceNumber: "0",
          };
        }
        
        const transaction = new TransactionBuilder(sourceAccount as any, {
          fee: "100",
          networkPassphrase,
        })
          .addOperation(operation as any)
          .setTimeout(30)
          .build();

        const result = await retryAsync(() => rpcServer.simulateTransaction(transaction));

        if (result && (result as any).retval) {
          const jsValue = scValToJs((result as any).retval);
          if (Array.isArray(jsValue)) {
            trackContractInteraction({
              method: "get_product_event_ids",
              durationMs: now() - startedAt,
              success: true,
              context: { productId, resultCount: jsValue.length },
            });
            return jsValue.map(Number);
          }
        }

        trackContractInteraction({
          method: "get_product_event_ids",
          durationMs: now() - startedAt,
          success: true,
          context: { productId, resultCount: 0 },
        });
        return [];
      } catch (error) {
        const normalized = normalizeContractClientError(error);
        trackContractInteraction({
          method: "get_product_event_ids",
          durationMs: now() - startedAt,
          success: false,
          errorMessage: normalized.message,
          context: { productId },
        });
        trackError(normalized, { source: "contractClient.get_product_event_ids", productId });
        throw normalized;
      }
    },

    async get_event(eventId: number): Promise<{
      event_id: number;
      product_id: string;
      actor: string;
      timestamp: number;
      event_type: string;
      note: string;
      data_hash?: string;
    } | null> {
      const startedAt = now();
      try {
        const operation = contract.call("get_event", xdr.ScVal.scvU64(xdr.Uint64.fromString(eventId.toString())));
        const networkPassphrase = network === "testnet" ? Networks.TESTNET : network === "mainnet" ? Networks.PUBLIC : Networks.TESTNET;
        const dummyAccount = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
        let sourceAccount: any;
        try {
          sourceAccount = await rpcServer.getAccount(dummyAccount);
        } catch {
          sourceAccount = {
            accountId: dummyAccount,
            sequenceNumber: "0",
          };
        }
        
        const transaction = new TransactionBuilder(sourceAccount as any, {
          fee: "100",
          networkPassphrase,
        })
          .addOperation(operation as any)
          .setTimeout(30)
          .build();

        const result = await retryAsync(() => rpcServer.simulateTransaction(transaction));

        if (!result || !(result as any).retval) {
          trackContractInteraction({
            method: "get_event",
            durationMs: now() - startedAt,
            success: true,
            context: { eventId, found: false },
          });
          return null;
        }

        const jsValue = scValToJs((result as any).retval);
        if (!jsValue || typeof jsValue !== "object") return null;

        const parsedEvent = {
          event_id: Number(jsValue.event_id || 0),
          product_id: scValToString(jsValue.product_id) || "",
          actor: scValToString(jsValue.actor) || "",
          timestamp: Number(jsValue.timestamp || 0),
          event_type: scValToString(jsValue.event_type) || "",
          note: scValToString(jsValue.note) || "",
          data_hash: jsValue.data_hash ? scValToString(jsValue.data_hash) : undefined,
        };

        trackContractInteraction({
          method: "get_event",
          durationMs: now() - startedAt,
          success: true,
          context: { eventId, found: true },
        });

        return parsedEvent;
      } catch (error) {
        const normalized = normalizeContractClientError(error);
        trackContractInteraction({
          method: "get_event",
          durationMs: now() - startedAt,
          success: false,
          errorMessage: normalized.message,
          context: { eventId },
        });
        trackError(normalized, { source: "contractClient.get_event", eventId });
        return null;
      }
    },
  };
}
