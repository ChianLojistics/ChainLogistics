"use client";

import Link from "next/link";
import { useState } from "react";
import { WalletStatus } from "../wallet";

export function NavBar() {
    const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

    return (
        <nav aria-label="Main navigation" className="border-b bg-white dark:bg-zinc-950 px-6 py-4">
            <div className="mx-auto max-w-7xl flex items-center justify-between">
                <Link href="/dashboard" className="text-xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
                    ChainLogistics
                </Link>

                <div className="flex items-center gap-8">
                    <div className="hidden md:flex items-center gap-6 text-sm font-medium text-zinc-600 dark:text-zinc-400">
                        <Link href="/dashboard" className="hover:text-blue-600 transition-colors">Dashboard</Link>
                        <Link href="/register" className="hover:text-blue-600 transition-colors">Register Product</Link>
                        <Link href="/tracking" className="hover:text-blue-600 transition-colors">Tracking</Link>
                    </div>

                    <WalletStatus />

                    <button
                        className="md:hidden text-zinc-600 hover:text-zinc-900"
                        onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
                        aria-label={mobileMenuOpen ? "Close menu" : "Open menu"}
                        aria-expanded={mobileMenuOpen}
                    >
                        <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                            {mobileMenuOpen ? (
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                            ) : (
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                            )}
                        </svg>
                    </button>
                </div>
            </div>

            {mobileMenuOpen && (
                <div className="md:hidden mt-4 border-t pt-4 pb-2 space-y-2">
                    <Link
                        href="/dashboard"
                        className="block px-3 py-2 text-sm font-medium text-zinc-600 hover:text-blue-600 hover:bg-zinc-50 rounded-md transition-colors"
                        onClick={() => setMobileMenuOpen(false)}
                    >
                        Dashboard
                    </Link>
                    <Link
                        href="/register"
                        className="block px-3 py-2 text-sm font-medium text-zinc-600 hover:text-blue-600 hover:bg-zinc-50 rounded-md transition-colors"
                        onClick={() => setMobileMenuOpen(false)}
                    >
                        Register Product
                    </Link>
                    <Link
                        href="/tracking"
                        className="block px-3 py-2 text-sm font-medium text-zinc-600 hover:text-blue-600 hover:bg-zinc-50 rounded-md transition-colors"
                        onClick={() => setMobileMenuOpen(false)}
                    >
                        Tracking
                    </Link>
                </div>
            )}
        </nav>
    );
}
