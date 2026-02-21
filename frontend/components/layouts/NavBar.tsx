"use client";

import Link from "next/link";
import { WalletConnect } from "../wallet";

export function NavBar() {
    return (
        <nav className="border-b bg-white dark:bg-zinc-950 px-6 py-4">
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

                    <WalletConnect />
                </div>
            </div>
        </nav>
    );
}
