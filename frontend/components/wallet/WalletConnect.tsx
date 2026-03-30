"use client";

import { useWalletStore } from "@/lib/state/wallet.store";
import { Button } from "@/components/ui/button";
import { Loader2, Wallet, CheckCircle2, Copy, LogOut, ExternalLink } from "lucide-react";
import { useCallback } from "react";
import { toast } from "sonner";

function truncateAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

export function WalletConnect() {
  const { status, publicKey, connect, disconnect, error } = useWalletStore();

  const handleConnect = async () => {
    try {
      await connect();
    } catch {
      // Error is stored in state and displayed below the button
    }
  };

  const handleCopyAddress = useCallback(() => {
    if (!publicKey) return;
    navigator.clipboard.writeText(publicKey);
    toast.success("Address copied to clipboard");
  }, [publicKey]);

  if (status === "connected" && publicKey) {
    return (
      <div className="flex items-center gap-1" role="group" aria-label="Wallet controls">
        <Button
          variant="outline"
          size="sm"
          onClick={handleCopyAddress}
          aria-label={`Wallet connected: ${publicKey}. Click to copy address.`}
          title="Copy wallet address"
          className="font-mono text-xs gap-1.5"
        >
          <CheckCircle2 className="h-3 w-3 text-green-500 shrink-0" aria-hidden="true" />
          {truncateAddress(publicKey)}
          <Copy className="h-3 w-3 text-muted-foreground" aria-hidden="true" />
        </Button>

        <Button
          variant="ghost"
          size="icon"
          onClick={() => disconnect()}
          aria-label="Disconnect wallet"
          title="Disconnect wallet"
          className="text-muted-foreground hover:text-destructive hover:bg-destructive/10"
        >
          <LogOut className="h-4 w-4" aria-hidden="true" />
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-end gap-1.5">
      <Button
        onClick={handleConnect}
        disabled={status === "connecting"}
        aria-label="Connect Freighter wallet"
        aria-busy={status === "connecting"}
        size="sm"
        className="gap-2"
      >
        {status === "connecting" ? (
          <>
            <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
            Connecting...
          </>
        ) : (
          <>
            <Wallet className="h-4 w-4" aria-hidden="true" />
            Connect Wallet
          </>
        )}
      </Button>

      {status === "error" && error && (
        <p className="text-xs text-destructive max-w-[200px] text-right" role="alert" aria-live="assertive">
          {error.toLowerCase().includes("not installed") ? (
            <>
              Freighter not found.{" "}
              <a
                href="https://www.freighter.app/"
                target="_blank"
                rel="noreferrer"
                className="underline hover:text-destructive/80 inline-flex items-center gap-0.5"
              >
                Install here
                <ExternalLink className="h-3 w-3" aria-hidden="true" />
              </a>
            </>
          ) : (
            error
          )}
        </p>
      )}
    </div>
  );
}
