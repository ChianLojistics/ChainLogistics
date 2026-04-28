"use client";

import { useEffect, useState, type ReactNode } from "react";
import { useWalletStore, type WalletState } from "@/lib/state/wallet.store";
import { useAppStore, type AppState } from "@/lib/state/app.store";
import { CONTRACT_CONFIG, validateContractConfig } from "@/lib/contract/config";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { i18n } from "@/lib/i18n";

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
  const initialize = useWalletStore((state: WalletState) => state.initialize);
  useEffect(() => {
    initialize();
  }, [initialize]);
  return null;
}

function NetworkInitializer() {
  const setNetwork = useAppStore((s: AppState) => s.setNetwork);
  useEffect(() => {
    setNetwork(CONTRACT_CONFIG.NETWORK);
  }, [setNetwork]);
  return null;
}

function I18nInitializer() {
  useEffect(() => {
    const apply = (lng: string) => {
      document.documentElement.dir = i18n.dir(lng);
      document.documentElement.lang = lng;
    };

    apply(i18n.resolvedLanguage || i18n.language || "en");
    i18n.on("languageChanged", apply);

    return () => {
      i18n.off("languageChanged", apply);
    };
  }, []);

  return null;
}

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <ErrorBoundary onReset={() => window.location.reload()}>
      <ContractConfigGuard />
      <NetworkInitializer />
      <WalletInitializer />
      <I18nInitializer />
      {children}
    </ErrorBoundary>
  );
}
