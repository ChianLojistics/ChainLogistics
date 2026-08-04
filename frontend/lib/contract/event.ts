/* eslint-disable @typescript-eslint/no-explicit-any */
import { xdr, Address, scValToNative } from "@stellar/stellar-sdk";
import { invokeContractWrite } from "@/lib/stellar/write";

const E2E_MOCKS_ENABLED = process.env.NEXT_PUBLIC_E2E_MOCKS === "true";

export type TrackingEventData = {
  productId: string;
  /** Short symbol, e.g. "shipped", "received", "processed" (<= 32 chars). */
  eventType: string;
  location: string;
  note?: string;
  /**
   * Optional 32-byte integrity hash as a hex string. If omitted, a SHA-256 of
   * the canonical event payload is computed in-browser.
   */
  dataHashHex?: string;
  /** Optional extra metadata (symbol -> string). */
  metadata?: Record<string, string>;
};

const ZERO_32 = new Uint8Array(32);

function hexToBytes32(hex: string): Uint8Array {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  if (clean.length !== 64 || /[^0-9a-fA-F]/.test(clean)) {
    throw new Error("dataHashHex must be 32 bytes (64 hex chars)");
  }
  const out = new Uint8Array(32);
  for (let i = 0; i < 32; i += 1) {
    out[i] = parseInt(clean.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}

/** SHA-256 of a canonical event payload, used when no explicit hash is given. */
async function deriveDataHash(data: TrackingEventData): Promise<Uint8Array> {
  const canonical = `${data.productId}|${data.eventType}|${data.location}|${data.note ?? ""}`;
  const cryptoObj = (globalThis as any).crypto;
  if (cryptoObj?.subtle) {
    const digest = await cryptoObj.subtle.digest(
      "SHA-256",
      new TextEncoder().encode(canonical)
    );
    return new Uint8Array(digest);
  }
  // Non-secure fallback (e.g. SSR without WebCrypto): a zero hash is still a
  // valid BytesN<32>; the on-chain record simply carries no integrity digest.
  return ZERO_32;
}

function metadataToScVal(metadata?: Record<string, string>): xdr.ScVal {
  if (!metadata) return xdr.ScVal.scvMap([]);
  const entries = Object.keys(metadata)
    .sort()
    .map(
      (key) =>
        new xdr.ScMapEntry({
          key: xdr.ScVal.scvSymbol(key),
          val: xdr.ScVal.scvString(metadata[key]),
        })
    );
  return xdr.ScVal.scvMap(entries);
}

/**
 * Add a tracking event on-chain via
 * `add_tracking_event(actor, product_id, event_type, location, data_hash, note, metadata)`.
 *
 * The connected wallet is the `actor` and signs the transaction, satisfying the
 * on-chain authorization check. Returns `{ hash, eventId }` where `eventId` is
 * the new event's u64 id (parsed from the return value; null if unavailable).
 */
export async function addTrackingEventOnChain(
  publicKey: string,
  data: TrackingEventData
): Promise<{ hash: string; eventId: number | null }> {
  if (!publicKey || !data.productId || !data.eventType) {
    throw new Error("Invalid tracking event parameters");
  }

  // E2E mode has no chain/Freighter behind it; return a deterministic fake
  // hash/eventId so the tracking flow reaches "Event Recorded!" without a
  // live network.
  if (E2E_MOCKS_ENABLED) {
    return {
      hash: `e2e-mock-tx-${data.productId}`.padEnd(64, "0").slice(0, 64),
      eventId: 1,
    };
  }

  const dataHash = data.dataHashHex
    ? hexToBytes32(data.dataHashHex)
    : await deriveDataHash(data);

  const args: xdr.ScVal[] = [
    new Address(publicKey).toScVal(),
    xdr.ScVal.scvString(data.productId),
    xdr.ScVal.scvSymbol(data.eventType),
    xdr.ScVal.scvString(data.location),
    xdr.ScVal.scvBytes(Buffer.from(dataHash)),
    xdr.ScVal.scvString(data.note ?? ""),
    metadataToScVal(data.metadata),
  ];

  const { hash, returnValue } = await invokeContractWrite({
    method: "add_tracking_event",
    args,
    publicKey,
  });

  let eventId: number | null = null;
  if (returnValue) {
    try {
      eventId = Number(scValToNative(returnValue));
    } catch {
      eventId = null;
    }
  }

  return { hash, eventId };
}
