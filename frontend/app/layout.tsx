import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { AppProviders } from "@/lib/state/providers";
import { Toaster } from "@/components/ui/sonner";
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
          {children}
          <Toaster />
        </AppProviders>
      </body>
    </html>
  );
}
