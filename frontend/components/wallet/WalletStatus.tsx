"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { ChevronDown, LogOut, Wallet, AlertTriangle } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { LoadingSpinner } from "@/components/ui/loading-spinner";
import { cn } from "@/lib/utils";
import { useWalletStore } from "@/lib/state/wallet.store";
import { useAppStore } from "@/lib/state/app.store";

function truncateAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

export function WalletStatus() {
  const { status, publicKey, connect, disconnect, error, network: walletNetwork } = useWalletStore();
  const appNetwork = useAppStore((s) => s.network);
  const network = walletNetwork ?? appNetwork;

  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);

  const isMainnet = network === "mainnet";
  const networkLabel = useMemo(() => (isMainnet ? "Mainnet" : "Testnet"), [isMainnet]);

  useEffect(() => {
    if (!open) return;

    const onMouseDown = (event: MouseEvent) => {
      const el = rootRef.current;
      if (!el) return;
      if (event.target instanceof Node && !el.contains(event.target)) {
        setOpen(false);
      }
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };

    window.addEventListener("mousedown", onMouseDown);
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("mousedown", onMouseDown);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  const handleConnect = async () => {
    try {
      await connect();
    } catch {
      // error is handled in store
    }
  };

  const handleDisconnect = async () => {
    await disconnect();
    setOpen(false);
  };

  const showErrorIndicator = status === "error" && !!error;

  return (
    <div className="relative flex items-center gap-2" ref={rootRef}>
      {status === "connected" && publicKey ? (
        <Badge
          variant="secondary"
          className={cn(
            "select-none border-transparent text-white",
            isMainnet
              ? "bg-blue-600 hover:bg-blue-600/90"
              : "bg-blue-500 hover:bg-blue-500/90"
          )}
          aria-label={`Network: ${networkLabel}`}
          title={`Network: ${networkLabel}`}
        >
          {networkLabel}
        </Badge>
      ) : null}

      {status !== "connected" || !publicKey ? (
        <div className="flex items-center gap-2">
          <Button
            onClick={handleConnect}
            disabled={status === "connecting"}
            aria-label="Connect Freighter wallet"
            aria-busy={status === "connecting"}
            size="sm"
            className="gap-2"
            variant="default"
            title={showErrorIndicator ? error ?? undefined : undefined}
          >
            {status === "connecting" ? (
              <>
                <LoadingSpinner size={16} aria-hidden="true" />
                Connecting...
              </>
            ) : (
              <>
                <Wallet className="h-4 w-4" aria-hidden="true" />
                Connect Wallet
              </>
            )}
          </Button>

          {showErrorIndicator && (
            <span
              className="inline-flex items-center"
              aria-live="polite"
              title={error ?? "Wallet connection error"}
            >
              <AlertTriangle
                className="h-4 w-4 text-destructive"
                aria-label={error ?? "Wallet connection error"}
              />
            </span>
          )}
        </div>
      ) : (
        <>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setOpen((v) => !v)}
            aria-haspopup="menu"
            aria-expanded={open}
            aria-label={`Wallet connected: ${publicKey}`}
            className="font-mono text-xs gap-1.5"
          >
            {truncateAddress(publicKey)}
            <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" aria-hidden="true" />
          </Button>

          {open && (
            <div
              role="menu"
              aria-label="Wallet menu"
              className="absolute right-0 top-full mt-2 min-w-[160px] rounded-md border bg-background shadow-md p-1 z-50"
            >
              <button
                type="button"
                role="menuitem"
                onClick={handleDisconnect}
                className="w-full flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground outline-none"
              >
                <LogOut className="h-4 w-4" aria-hidden="true" />
                Disconnect
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
