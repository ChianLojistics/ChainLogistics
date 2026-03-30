"use client";

import { useEffect, useState } from "react";
import { useWalletStore } from "@/lib/state/wallet.store";
import { useAppStore } from "@/lib/state/app.store";
import { CONTRACT_CONFIG, validateContractConfig } from "@/lib/contract/config";
import { ErrorBoundary } from "@/components/ErrorBoundary";

function ContractConfigGuard() {
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    try {
      validateContractConfig();
    } catch (err) {
      const nextError = err instanceof Error ? err : new Error(String(err));
      Promise.resolve().then(() => setError(nextError));
    }
  }, []);

  if (error) throw error;
  return null;
}

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
      <ContractConfigGuard />
      <NetworkInitializer />
      <WalletInitializer />
      {children}
    </ErrorBoundary>
  );
}
