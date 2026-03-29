import { vi } from "vitest";
import { config } from "@testing-library/react";

// Extend global types for our test environment
declare global {
  var IntersectionObserver: any;
  var ResizeObserver: any;
  var WebSocket: any;
  var URL: any;
  var Blob: any;
  var File: any;
  var FileReader: any;
  var performance: Performance;
  var crypto: Crypto;
}

// Mock Next.js router
vi.mock("next/router", () => ({
  useRouter() {
    return {
      route: "/",
      pathname: "/",
      query: "",
      asPath: "",
      push: vi.fn(),
      pop: vi.fn(),
      reload: vi.fn(),
      back: vi.fn(),
      prefetch: vi.fn(),
      beforePopState: vi.fn(),
      events: {
        on: vi.fn(),
        off: vi.fn(),
        emit: vi.fn(),
      },
    };
  },
}));

// Mock Next.js navigation
vi.mock("next/navigation", () => ({
  useRouter() {
    return {
      push: vi.fn(),
      replace: vi.fn(),
      refresh: vi.fn(),
      back: vi.fn(),
      forward: vi.fn(),
      prefetch: vi.fn(),
    };
  },
  useSearchParams() {
    return new URLSearchParams();
  },
  usePathname() {
    return "/";
  },
}));

// Mock window.matchMedia
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock IntersectionObserver
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Mock fetch
global.fetch = vi.fn();

// Mock WebSocket
global.WebSocket = vi.fn().mockImplementation(() => ({
  close: vi.fn(),
  send: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  readyState: 1,
  CONNECTING: 0,
  OPEN: 1,
  CLOSING: 2,
  CLOSED: 3,
}));

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
vi.stubGlobal("localStorage", localStorageMock);

// Mock sessionStorage
const sessionStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
vi.stubGlobal("sessionStorage", sessionStorageMock);

// Mock console methods in tests
global.console = {
  ...console,
  warn: vi.fn(),
  error: vi.fn(),
  log: vi.fn(),
};

// Configure testing library
config({
  testIdAttribute: "data-testid",
});

// Mock environment variables
vi.stubEnv("NEXT_PUBLIC_CONTRACT_ID", "CBUWSKT2UGOAXK4ZREVDJV5XHSYB42PZ3CERU2ZFUTUMAZLJEHNZIECA");
vi.stubEnv("NEXT_PUBLIC_STELLAR_NETWORK", "testnet");
vi.stubEnv("NEXT_PUBLIC_STELLAR_RPC_URL", "https://soroban-testnet.stellar.org");
vi.stubEnv("NEXT_PUBLIC_API_URL", "http://localhost:3001");
vi.stubEnv("NEXT_PUBLIC_WS_URL", "ws://localhost:3001");

// Mock performance API
Object.defineProperty(global, "performance", {
  value: {
    ...global.performance,
    now: vi.fn(() => Date.now()),
    mark: vi.fn(),
    measure: vi.fn(),
    getEntriesByName: vi.fn(() => []),
    getEntriesByType: vi.fn(() => []),
  },
  writable: true,
});

// Mock crypto API
Object.defineProperty(global, "crypto", {
  value: {
    randomUUID: vi.fn(() => "mock-uuid"),
    getRandomValues: vi.fn(() => new Uint32Array(1)),
  },
  writable: true,
});

// Mock URL constructor
global.URL = class URL {
  constructor(url: string, base?: string) {
    this.href = url;
    this.origin = "http://localhost:3000";
    this.protocol = "http:";
    this.host = "localhost:3000";
    this.hostname = "localhost";
    this.port = "3000";
    this.pathname = "/";
    this.search = "";
    this.hash = "";
  }
  href: string;
  origin: string;
  protocol: string;
  host: string;
  hostname: string;
  port: string;
  pathname: string;
  search: string;
  hash: string;
  toString() {
    return this.href;
  }
};

// Mock Blob
global.Blob = class Blob {
  constructor(content: any[], options?: BlobPropertyBag) {
    this.size = content.reduce((acc, part) => acc + (part?.length || 0), 0);
    this.type = options?.type || "";
  }
  size: number;
  type: string;
  arrayBuffer() {
    return Promise.resolve(new ArrayBuffer(0));
  }
  text() {
    return Promise.resolve("");
  }
  stream() {
    return new ReadableStream();
  }
  slice() {
    return new Blob([]);
  }
};

// Mock File
global.File = class File extends Blob {
  constructor(content: any[], name: string, options?: FilePropertyBag) {
    super(content, options);
    this.name = name;
    this.lastModified = Date.now();
  }
  name: string;
  lastModified: number;
};

// Mock FileReader
global.FileReader = class FileReader {
  result: string | ArrayBuffer | null = null;
  error: any = null;
  readyState: number = 0;
  EMPTY: number = 0;
  LOADING: number = 1;
  DONE: number = 2;
  
  onload: ((event: any) => void) | null = null;
  onerror: ((event: any) => void) | null = null;
  onloadend: ((event: any) => void) | null = null;
  
  readAsDataURL(blob: Blob) {
    setTimeout(() => {
      this.result = "data:image/png;base64,mock-data";
      this.readyState = this.DONE;
      this.onload?.({ target: this } as any);
    }, 0);
  }
  
  readAsText(blob: Blob) {
    setTimeout(() => {
      this.result = "mock-text";
      this.readyState = this.DONE;
      this.onload?.({ target: this } as any);
    }, 0);
  }
  
  readAsArrayBuffer(blob: Blob) {
    setTimeout(() => {
      this.result = new ArrayBuffer(0);
      this.readyState = this.DONE;
      this.onload?.({ target: this } as any);
    }, 0);
  }
  
  abort() {
    this.readyState = this.DONE;
  }
};
