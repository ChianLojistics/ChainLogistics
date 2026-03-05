"use client";

import { useEffect } from "react";
import { useWalletStore } from "@/lib/state/wallet.store";
import { useAppStore } from "@/lib/state/app.store";
import { CONTRACT_CONFIG } from "@/lib/contract/config";
import { ErrorBoundary } from "@/components/ErrorBoundary";

function WalletInitializer() {
  const initialize = useWalletStore((state) => state.initialize);
  useEffect(() => {
    initialize();
  }, [initialize]);
  return null;
}

function NetworkInitializer() {
  const setNetwork = useAppStore((s) => s.setNetwork);
  useEffect(() => {
    setNetwork(CONTRACT_CONFIG.NETWORK);
  }, [setNetwork]);
  return null;
}

export function AppProviders({ children }: { children: React.ReactNode }) {
  return (
    <ErrorBoundary onReset={() => window.location.reload()}>
      <NetworkInitializer />
      <WalletInitializer />
      {children}
    </ErrorBoundary>
  );
}
