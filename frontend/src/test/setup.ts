import "@testing-library/jest-dom";
import { vi } from "vitest";

Object.defineProperty(HTMLCanvasElement.prototype, "getContext", {
  value: vi.fn(() => null),
  configurable: true,
  writable: true,
});

// Mock Next.js router
vi.mock("next/navigation", () => ({
  useRouter() {
    return {
      push: vi.fn(),
      replace: vi.fn(),
      prefetch: vi.fn(),
    };
  },
  useSearchParams() {
    return new URLSearchParams();
  },
  usePathname() {
    return "";
  },
}));

// Mock next/link
vi.mock("next/link", () => ({
  default: ({
    children,
    href,
    onClick,
    className,
    ...props
  }: {
    children: React.ReactNode;
    href: string;
    onClick?: React.MouseEventHandler;
    className?: string;
  }) =>
    React.createElement(
      "a",
      { href, onClick, className, ...props },
      children
    ),
}));

// Mock next-themes
vi.mock("next-themes", () => ({
  useTheme: () => ({ theme: "light", setTheme: vi.fn() }),
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en", changeLanguage: vi.fn() },
  }),
  Trans: ({ children }: { children: React.ReactNode }) => children,
  initReactI18next: { type: "3rdParty", init: vi.fn() },
}));

// Mock Stellar SDK
vi.mock("@stellar/stellar-sdk", () => ({
  default: {},
  Horizon: { Server: vi.fn() },
  SorobanRpc: { Server: vi.fn() },
  nativeToScVal: vi.fn(),
  scValToNative: vi.fn(),
  Keypair: { fromSecret: vi.fn(), random: vi.fn() },
  Networks: { PUBLIC: "public", TESTNET: "testnet", FUTURENET: "futurenet" },
  TransactionBuilder: vi.fn(),
  Operation: { payment: vi.fn() },
  Asset: { native: vi.fn() },
  BASE_FEE: "100",
}));

// Mock Freighter API
vi.mock("@stellar/freighter-api", () => ({
  isConnected: vi.fn().mockResolvedValue({ isConnected: true }),
  getPublicKey: vi.fn().mockResolvedValue("GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890"),
  signTransaction: vi.fn().mockResolvedValue({ signedTxXdr: "AAAA..." }),
  signBlob: vi.fn().mockResolvedValue({ signedBlob: "signed..." }),
}));

// Mock blockchain providers
vi.mock("@/lib/blockchain/providers/stellar", () => ({
  StellarProvider: vi.fn().mockImplementation(() => ({
    connect: vi.fn(),
    disconnect: vi.fn(),
    signAndSendTransaction: vi.fn(),
    getPublicKey: vi.fn().mockReturnValue("GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890"),
    isConnected: vi.fn().mockReturnValue(true),
  })),
}));

// Mock contract/events module
vi.mock("@/lib/contract/events", () => ({
  fetchProductEventsPage: vi.fn(),
  getRelativeTime: vi.fn((ts: number) => `${Math.floor((Date.now() / 1000 - ts) / 86400)} days ago`),
  formatEventTimestamp: vi.fn((ts: number) => new Date(ts * 1000).toLocaleString()),
}));

// Mock contract/product module
vi.mock("@/lib/contract/product", () => ({
  registerProduct: vi.fn(),
  getProduct: vi.fn(),
  getProducts: vi.fn(),
}));

// Mock websocket client
vi.mock("@/lib/websocket/client", () => ({
  WebSocketClient: vi.fn().mockImplementation(() => ({
    connect: vi.fn(),
    disconnect: vi.fn(),
    subscribe: vi.fn(),
    unsubscribe: vi.fn(),
    on: vi.fn(),
    off: vi.fn(),
  })),
}));

// Mock toast store
vi.mock("@/lib/toast/store", () => ({
  useToastStore: vi.fn().mockReturnValue({
    toasts: [],
    position: "top-right",
    addToast: vi.fn(),
    removeToast: vi.fn(),
    clearToasts: vi.fn(),
  }),
  type: { ToastType: {} },
}));

// Mock sonner
vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
  Toaster: ({ ...props }: Record<string, unknown>) =>
    React.createElement("div", { ...props }, null),
}));

// Mock wallet store
vi.mock("@/lib/state/wallet.store", () => ({
  useWalletStore: vi.fn().mockReturnValue({
    status: "disconnected",
    publicKey: null,
    connect: vi.fn(),
    disconnect: vi.fn(),
    error: null,
  }),
}));

// Mock app store
vi.mock("@/lib/state/app.store", () => ({
  useAppStore: vi.fn().mockReturnValue({
    isOnline: true,
    isSidebarOpen: false,
    toggleSidebar: vi.fn(),
  }),
}));

// Mock analytics
vi.mock("@/lib/analytics", () => ({
  trackEvent: vi.fn(),
  AnalyticsProvider: ({ children }: { children: React.ReactNode }) => children,
}));

// Mock environment variables
process.env.NEXT_PUBLIC_STELLAR_NETWORK = "testnet";
process.env.NEXT_PUBLIC_RPC_URL = "https://soroban-testnet.stellar.org";
process.env.NEXT_PUBLIC_APP_URL = "http://localhost:3000";

// Global test utilities
global.matchMedia = global.matchMedia || function() {
  return {
    matches: false,
    addListener: function() {},
    removeListener: function() {},
  };
};

// Mock IntersectionObserver
class MockIntersectionObserver {
  readonly root: Element | null = null;
  readonly rootMargin: string = "";
  readonly thresholds: ReadonlyArray<number> = [];
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  takeRecords = vi.fn();
}

Object.defineProperty(window, "IntersectionObserver", {
  writable: true,
  configurable: true,
  value: MockIntersectionObserver,
});

Object.defineProperty(global, "IntersectionObserver", {
  writable: true,
  configurable: true,
  value: MockIntersectionObserver,
});

// Import React for JSX in mocks
import React from "react";
