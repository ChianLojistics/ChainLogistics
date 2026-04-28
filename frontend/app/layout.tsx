import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { AppProviders } from "@/lib/state/providers";
import { MonitoringBootstrap, PerformanceBudgetAlerts } from "@/components/analytics";
import { Toaster } from "@/components/ui/sonner";
import { ToastContainer } from "@/components/ui/ToastContainer";
import { SkipToContentLink } from "@/components/SkipToContentLink";
import "./globals.css";

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

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <AppProviders>
          <SkipToContentLink />
          <MonitoringBootstrap />
          <PerformanceBudgetAlerts />
          {children}
          <Toaster />
          <ToastContainer />
        </AppProviders>
      </body>
    </html>
  );
}
