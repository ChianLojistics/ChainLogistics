/* eslint-disable @typescript-eslint/no-explicit-any */
import { xdr, Address, nativeToScVal } from "@stellar/stellar-sdk";
import { invokeContractWrite } from "@/lib/stellar/write";

export type ProductData = {
  id: string;
  name: string;
  origin: string;
  description?: string;
  category: string;
};

/**
 * Encode a Soroban `#[contracttype]` struct as an ScVal map.
 *
 * Struct fields are represented as an ScMap keyed by the field-name symbol,
 * and the host requires entries in sorted key order — so we sort by symbol.
 */
function structToScVal(fields: Record<string, xdr.ScVal>): xdr.ScVal {
  const entries = Object.keys(fields)
    .sort()
    .map(
      (key) =>
        new xdr.ScMapEntry({
          key: xdr.ScVal.scvSymbol(key),
          val: fields[key],
        })
    );
  return xdr.ScVal.scvMap(entries);
}

const strVal = (s: string): xdr.ScVal => xdr.ScVal.scvString(s);
const emptyVec = (): xdr.ScVal => xdr.ScVal.scvVec([]);
const emptyMap = (): xdr.ScVal => xdr.ScVal.scvMap([]);

/**
 * Build the `ProductConfig` argument for `register_product`.
 *
 * Field order/types mirror the Rust `ProductConfig` struct:
 *   id, name, description, origin_location, category: String
 *   tags: Vec<String>          (empty for the MVP form)
 *   certifications: Vec<BytesN<32>>   (empty)
 *   media_hashes: Vec<BytesN<32>>     (empty)
 *   custom: Map<Symbol, String>       (empty)
 */
function buildProductConfig(data: ProductData): xdr.ScVal {
  return structToScVal({
    id: strVal(data.id),
    name: strVal(data.name),
    description: strVal(data.description ?? ""),
    origin_location: strVal(data.origin),
    category: strVal(data.category),
    tags: emptyVec(),
    certifications: emptyVec(),
    media_hashes: emptyVec(),
    custom: emptyMap(),
  });
}

/**
 * Register a product on-chain via `register_product(owner, config)`.
 *
 * `owner` is the connected wallet, which also signs the transaction, so its
 * `require_auth()` is satisfied by the envelope signature. Returns the
 * confirmed transaction hash.
 */
export async function registerProductOnChain(
  publicKey: string,
  data: ProductData
): Promise<string> {
  if (!publicKey || !data.id) {
    throw new Error("Invalid contract parameters");
  }

  const ownerScVal = new Address(publicKey).toScVal();
  const configScVal = buildProductConfig(data);

  const { hash } = await invokeContractWrite({
    method: "register_product",
    args: [ownerScVal, configScVal],
    publicKey,
  });

  return hash;
}

// Re-export so callers can encode addresses without importing the SDK directly.
export { nativeToScVal };
