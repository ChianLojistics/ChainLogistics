import { useAppStore } from "@/lib/state/app.store";
import { HORIZON_URL_BY_NETWORK } from "@/lib/stellar";

export function useStellar() {
  const network = useAppStore((s) => s.network);
  return {
    network,
    horizonUrl: HORIZON_URL_BY_NETWORK[network],
  };
}
