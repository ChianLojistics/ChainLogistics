import { Contract, xdr, rpc, TransactionBuilder, Networks } from "@stellar/stellar-sdk";
import type { StellarNetwork } from "./networks";
import { CONTRACT_CONFIG } from "@/lib/contract/config";

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
    throw new Error("Contract ID not configured");
  }

  const rpcServer = new rpc.Server(rpcUrl, { allowHttp: true });
  const contract = new Contract(contractId);

  return {
    async ping(): Promise<string> {
      return "ok";
    },

    async get_product_event_ids(productId: string): Promise<number[]> {
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

        const result = await rpcServer.simulateTransaction(transaction);

        if (result && (result as any).retval) {
          const jsValue = scValToJs((result as any).retval);
          if (Array.isArray(jsValue)) {
            return jsValue.map(Number);
          }
        }
        return [];
      } catch (error) {
        console.error("Failed to get product event IDs:", error);
        throw error;
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

        const result = await rpcServer.simulateTransaction(transaction);

        if (!result || !(result as any).retval) return null;

        const jsValue = scValToJs((result as any).retval);
        if (!jsValue || typeof jsValue !== "object") return null;

        return {
          event_id: Number(jsValue.event_id || 0),
          product_id: scValToString(jsValue.product_id) || "",
          actor: scValToString(jsValue.actor) || "",
          timestamp: Number(jsValue.timestamp || 0),
          event_type: scValToString(jsValue.event_type) || "",
          note: scValToString(jsValue.note) || "",
          data_hash: jsValue.data_hash ? scValToString(jsValue.data_hash) : undefined,
        };
      } catch (error) {
        console.error(`Failed to get event ${eventId}:`, error);
        return null;
      }
    },
  };
}
