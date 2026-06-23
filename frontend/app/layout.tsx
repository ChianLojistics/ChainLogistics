import type { Metadata } from "next";
import dynamic from "next/dynamic";
import { Geist, Geist_Mono } from "next/font/google";
import { AppProviders } from "@/lib/state/providers";
import { Toaster } from "@/components/ui/sonner";
import { ToastContainer } from "@/components/ui/ToastContainer";
import { SkipToContentLink } from "@/components/SkipToContentLink";
import "./globals.css";

const enablePerformanceToasts = process.env.NODE_ENV !== "production";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "ChainLojistic - Transparent Supply Chain Tracking on Blockchain",
  description:
    "Track products from origin to consumer with immutable blockchain records. Verify authenticity, combat counterfeits, and build trust through tamper-proof supply chain tracking on Stellar blockchain.",
  keywords: [
    "supply chain",
    "blockchain",
    "transparency",
    "traceability",
    "Stellar",
    "Soroban",
    "product tracking",
    "counterfeit prevention",
    "verification",
  ],
  authors: [{ name: "ChainLojistic" }],
  openGraph: {
    title: "ChainLojistic - Transparent Supply Chain Tracking",
    description:
      "Track products from origin to consumer with immutable blockchain records.",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "ChainLojistic - Transparent Supply Chain Tracking",
    description:
      "Track products from origin to consumer with immutable blockchain records.",
  },
  other: {
    "Content-Security-Policy":
      "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https:; connect-src 'self' https://horizon-testnet.stellar.org https://soroban-testnet.stellar.org https://nominatim.openstreetmap.org; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
  },
};

const MonitoringBootstrap = dynamic(
  () =>
    import("@/components/analytics/MonitoringBootstrap").then((m) => ({
      default: m.MonitoringBootstrap,
    }))
);

const PerformanceBudgetAlerts = dynamic(
  () =>
    import("@/components/analytics/PerformanceBudgetAlerts").then((m) => ({
      default: m.PerformanceBudgetAlerts,
    }))
);

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://horizon-testnet.stellar.org" />
        <link rel="preconnect" href="https://soroban-testnet.stellar.org" />
      </head>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <AppProviders>
          <SkipToContentLink />
          <MonitoringBootstrap />
          {enablePerformanceToasts ? <PerformanceBudgetAlerts /> : null}
          {children}
          <Toaster />
          <ToastContainer />
        </AppProviders>
      </body>
    </html>
  );
}
