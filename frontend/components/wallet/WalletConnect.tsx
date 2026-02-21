"use client";

import { useWalletStore } from "@/lib/state/wallet.store";
import { useState } from "react";

export function WalletConnect() {
    const { status, publicKey, connect, disconnect, error } = useWalletStore();
    const [isHovered, setIsHovered] = useState(false);

    const handleConnect = async () => {
        try {
            await connect();
        } catch (err) {
            console.error("Connection failed", err);
        }
    };

    const truncatedAddress = publicKey
        ? `${publicKey.slice(0, 6)}...${publicKey.slice(-4)}`
        : "";

    if (status === "connected") {
        return (
            <div className="flex items-center gap-4">
                <button
                    onClick={() => disconnect()}
                    onMouseEnter={() => setIsHovered(true)}
                    onMouseLeave={() => setIsHovered(false)}
                    className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-100 dark:bg-gray-800 text-sm font-medium transition-all hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20"
                >
                    <span className="w-2 h-2 rounded-full bg-green-500" />
                    {isHovered ? "Disconnect" : truncatedAddress}
                </button>
            </div>
        );
    }

    return (
        <div className="flex flex-col items-end gap-2">
            <button
                disabled={status === "connecting"}
                onClick={handleConnect}
                className="px-6 py-2 rounded-lg bg-blue-600 text-white text-sm font-semibold transition-all hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
                {status === "connecting" ? (
                    <>
                        <svg className="animate-spin h-4 w-4 text-white" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                        </svg>
                        Connecting...
                    </>
                ) : (
                    "Connect Wallet"
                )}
            </button>

            {status === "error" && error && (
                <p className="text-xs text-red-500 mt-1">
                    {error.includes("not installed") ? (
                        <span>
                            Freighter not found.{" "}
                            <a
                                href="https://www.freighter.app/"
                                target="_blank"
                                rel="noreferrer"
                                className="underline hover:text-red-600"
                            >
                                Install here
                            </a>
                        </span>
                    ) : error}
                </p>
            )}
        </div>
    );
}
