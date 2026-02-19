export function shortenPublicKey(publicKey: string, chars = 4) {
  if (publicKey.length <= chars * 2) return publicKey;
  return `${publicKey.slice(0, chars)}…${publicKey.slice(-chars)}`;
}
