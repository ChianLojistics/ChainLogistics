/* eslint-disable @typescript-eslint/no-explicit-any */
import {
  Contract,
  xdr,
  rpc,
  TransactionBuilder,
  Networks,
  BASE_FEE,
} from "@stellar/stellar-sdk";
import { CONTRACT_CONFIG, validateContractConfig } from "@/lib/contract/config";
import { getFreighterNetwork, signWithFreighter } from "@/lib/stellar/wallet";
import { trackContractInteraction, trackError } from "@/lib/analytics";

/**
 * Error raised while building, signing, or submitting a contract write.
 * `userMessage` is safe to surface directly in the UI.
 */
export class ContractWriteError extends Error {
  userMessage: string;
  constructor(message: string, userMessage?: string, cause?: unknown) {
    super(message);
    this.name = "ContractWriteError";
    this.userMessage = userMessage ?? message;
    if (cause !== undefined) (this as any).cause = cause;
  }
}

function networkPassphrase(network: string): string {
  switch (network) {
    case "mainnet":
      return Networks.PUBLIC;
    case "futurenet":
      return Networks.FUTURENET;
    case "testnet":
    default:
      return Networks.TESTNET;
  }
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Guard against the most common failure: the wallet is pointed at a different
 * network than the app. Freighter would sign a transaction for the wrong
 * passphrase and the submit would fail with a confusing error.
 *
 * If Freighter's network can't be determined (older API / locked), we skip the
 * check rather than block the user.
 */
async function assertWalletNetworkMatches(appNetwork: string): Promise<void> {
  const walletNetwork = await getFreighterNetwork();
  if (walletNetwork && walletNetwork !== appNetwork) {
    throw new ContractWriteError(
      `Wallet network mismatch: Freighter is on ${walletNetwork}, app expects ${appNetwork}`,
      `Your wallet is set to ${walletNetwork}, but this app is running on ${appNetwork}. Switch Freighter to ${appNetwork} and try again.`
    );
  }
}

export type ContractWriteResult = {
  hash: string;
  returnValue: xdr.ScVal | null;
};

/**
 * Build, simulate/prepare, sign (via Freighter), submit, and confirm a single
 * contract invocation.
 *
 * The connected wallet (`publicKey`) is used as the transaction source, so any
 * `require_auth()` on that same address is satisfied by the envelope signature —
 * no separate Soroban auth-entry signing is needed for the MVP methods
 * (`register_product`, `add_tracking_event`), where the authorizer is the
 * invoker.
 */
export async function invokeContractWrite(params: {
  method: string;
  args: xdr.ScVal[];
  publicKey: string;
  /** Wait for on-chain confirmation (default true). */
  waitForConfirmation?: boolean;
}): Promise<ContractWriteResult> {
  const { method, args, publicKey } = params;
  const waitForConfirmation = params.waitForConfirmation ?? true;
  const startedAt = Date.now();

  validateContractConfig();

  const { CONTRACT_ID, NETWORK, RPC_URL } = CONTRACT_CONFIG;
  const passphrase = networkPassphrase(NETWORK);

  try {
    await assertWalletNetworkMatches(NETWORK);

    const server = new rpc.Server(RPC_URL, { allowHttp: true });
    const contract = new Contract(CONTRACT_ID);

    let source;
    try {
      source = await server.getAccount(publicKey);
    } catch (err) {
      throw new ContractWriteError(
        `Failed to load source account ${publicKey}`,
        "Your account was not found on the network. Fund it with testnet XLM (friendbot) and try again.",
        err
      );
    }

    const built = new TransactionBuilder(source, {
      fee: BASE_FEE,
      networkPassphrase: passphrase,
    })
      .addOperation(contract.call(method, ...args))
      .setTimeout(180)
      .build();

    // prepareTransaction runs simulation, applies the Soroban footprint,
    // resource fees, and auth. Throws if the simulation reverts.
    let prepared;
    try {
      prepared = await server.prepareTransaction(built);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw new ContractWriteError(
        `Simulation failed for ${method}: ${message}`,
        `The transaction was rejected during simulation. ${extractContractError(message)}`,
        err
      );
    }

    const signedXdr = await signWithFreighter(prepared.toXDR(), passphrase);
    const signedTx = TransactionBuilder.fromXDR(signedXdr, passphrase);

    const sendResponse = await server.sendTransaction(signedTx as any);
    if (sendResponse.status === "ERROR") {
      const detail = safeStringify((sendResponse as any).errorResult);
      throw new ContractWriteError(
        `sendTransaction returned ERROR for ${method}: ${detail}`,
        "The network rejected the transaction. Please try again."
      );
    }

    const hash = sendResponse.hash;

    if (!waitForConfirmation) {
      trackContractInteraction({
        method,
        durationMs: Date.now() - startedAt,
        success: true,
        context: { hash, confirmed: false },
      });
      return { hash, returnValue: null };
    }

    // Poll for confirmation.
    const maxAttempts = 30;
    let attempt = 0;
    let getResponse = await server.getTransaction(hash);
    while (getResponse.status === "NOT_FOUND" && attempt < maxAttempts) {
      attempt += 1;
      await sleep(1500);
      getResponse = await server.getTransaction(hash);
    }

    if (getResponse.status === "SUCCESS") {
      trackContractInteraction({
        method,
        durationMs: Date.now() - startedAt,
        success: true,
        context: { hash, confirmed: true },
      });
      return { hash, returnValue: (getResponse as any).returnValue ?? null };
    }

    if (getResponse.status === "NOT_FOUND") {
      // Submitted but not yet visible — hand back the hash so the UI can link out.
      trackContractInteraction({
        method,
        durationMs: Date.now() - startedAt,
        success: true,
        context: { hash, confirmed: false, timedOut: true },
      });
      return { hash, returnValue: null };
    }

    throw new ContractWriteError(
      `Transaction ${hash} failed with status ${getResponse.status}`,
      "The transaction was submitted but failed on-chain. Check the explorer for details."
    );
  } catch (error) {
    if (!(error instanceof ContractWriteError)) {
      trackError(error, { source: `write.${method}` });
    }
    trackContractInteraction({
      method,
      durationMs: Date.now() - startedAt,
      success: false,
      errorMessage: error instanceof Error ? error.message : String(error),
    });
    throw error instanceof ContractWriteError
      ? error
      : new ContractWriteError(
          error instanceof Error ? error.message : String(error),
          "Something went wrong while submitting the transaction.",
          error
        );
  }
}

/** Best-effort surfacing of a Soroban contract error code from a message. */
function extractContractError(message: string): string {
  const match = message.match(/Error\(Contract, #(\d+)\)/);
  if (match) return `Contract error #${match[1]}.`;
  if (message.toLowerCase().includes("existingvalue") || message.includes("#3")) {
    return "This product ID may already exist.";
  }
  return "";
}

function safeStringify(value: unknown): string {
  try {
    return typeof value === "string" ? value : JSON.stringify(value);
  } catch {
    return String(value);
  }
}
