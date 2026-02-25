"use client";

import { useEffect } from "react";
import { useWalletStore } from "@/lib/state/wallet.store";

function WalletInitializer() {
  const initialize = useWalletStore((state) => state.initialize);
  useEffect(() => {
    initialize();
  }, [initialize]);
  return null;
}

export function AppProviders({ children }: { children: React.ReactNode }) {
  return (
    <>
      <WalletInitializer />
      {children}
    </>
  );
}
