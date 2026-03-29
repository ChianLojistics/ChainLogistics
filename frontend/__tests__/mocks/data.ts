import type { Product, TrackingEvent } from "@/lib/types";

export const mockProduct: Product = {
  id: "PROD-001",
  name: "Organic Coffee Beans",
  description: "Premium organic coffee beans from Ethiopia",
  category: "Beverages",
  origin: {
    location: "Ethiopia",
    coordinates: {
      latitude: 9.145,
      longitude: 40.4897,
    },
    timestamp: 1640995200, // 2022-01-01
    verified: true,
  },
  owner: "GBX2NJBSK3LUMRXF24F6EBV4TQJQU5WIK6HM4YJOTGZLJLOK5ZU6XHSY",
  current_holder: "GBX2NJBSK3LUMRXF24F6EBV4TQJQU5WIK6HM4YJOTGZLJLOK5ZU6XHSY",
  status: "active",
  created_at: 1640995200,
  updated_at: 1640995200,
  certifications: [
    {
      id: "CERT-001",
      name: "USDA Organic",
      issuer: "USDA",
      issued_at: 1640995200,
      expires_at: 1704067200,
      verified: true,
    },
  ],
  tags: ["organic", "fair-trade", "premium"],
  media_hashes: [
    "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
    "QmYXotypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWyapu",
  ],
  custom_fields: {
    roast_level: "medium",
    flavor_notes: ["chocolate", "citrus", "caramel"],
    altitude: "1500-2000m",
  },
  active: true,
};

export const mockTrackingEvents: TrackingEvent[] = [
  {
    id: "EVENT-001",
    product_id: "PROD-001",
    actor: "GBX2NJBSK3LUMRXF24F6EBV4TQJQU5WIK6HM4YJOTGZLJLOK5ZU6XHSY",
    event_type: "created",
    timestamp: 1640995200,
    location: {
      address: "Addis Ababa, Ethiopia",
      coordinates: {
        latitude: 9.145,
        longitude: 40.4897,
      },
    },
    metadata: {
      temperature: 22,
      humidity: 65,
      batch_number: "BATCH-001",
    },
    note: "Product created and registered on blockchain",
    verified: true,
    transaction_hash: "0x1234567890abcdef",
  },
  {
    id: "EVENT-002",
    product_id: "PROD-001",
    actor: "GBX2NJBSK3LUMRXF24F6EBV4TQJQU5WIK6HM4YJOTGZLJLOK5ZU6XHSY",
    event_type: "harvested",
    timestamp: 1641081600,
    location: {
      address: "Addis Ababa, Ethiopia",
      coordinates: {
        latitude: 9.145,
        longitude: 40.4897,
      },
    },
    metadata: {
      temperature: 25,
      humidity: 70,
      harvest_method: "hand-picked",
      quality_grade: "A",
    },
    note: "Coffee beans harvested with quality grade A",
    verified: true,
    transaction_hash: "0x1234567890abcdeg",
  },
  {
    id: "EVENT-003",
    product_id: "PROD-001",
    actor: "GBX2NJBSK3LUMRXF24F6EBV4TQJQU5WIK6HM4YJOTGZLJLOK5ZU6XHSY",
    event_type: "processed",
    timestamp: 1641168000,
    location: {
      address: "Processing Facility, Ethiopia",
      coordinates: {
        latitude: 9.0,
        longitude: 40.0,
      },
    },
    metadata: {
      processing_method: "washed",
      drying_time: 72,
      moisture_content: 11.5,
    },
    note: "Coffee beans processed using washed method",
    verified: true,
    transaction_hash: "0x1234567890abcdefh",
  },
  {
    id: "EVENT-004",
    product_id: "PROD-001",
    actor: "GD5DJEUPDFV5X5LYLYY ChY ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly ly lyivedxjQkDDP1m cy ly ly ly ly ly ly ly ly ly ly ly ly ly ly lyivedxjQkDDP1m cy ly ly lyives",
    event_type: "shipped",
    timestamp: 1641254400,
    location: {
      address: "Port of Djibouti, Djibouti",
      coordinates: {
        latitude: 11.595,
        longitude: 43.148,
      },
    },
    metadata: {
      shipping_method: "container",
      container_number: "CONT-123456",
      temperature: 20,
      humidity: 60,
    },
    note: "Product shipped to international destination",
    verified: true,
    transaction_hash: "0x1234567890abcdefi",
  },
  {
    id: "EVENT-005",
    product_id: "PROD-001",
    actor: "GBX2NJBSK3LUMRXF24F6EBV4TQJQU5WIK6HM4YJOTGZLJLOK5ZU6XHSY",
    event_type: "received",
    timestamp: 1641340800,
    location: {
      address: "New York, USA",
      coordinates: {
        latitude: 40.7128,
        longitude: -74.0060,
      },
    },
    metadata: {
      receiving_facility: "Warehouse-001",
      inspection_passed: true,
      storage_conditions: "optimal",
    },
    note: "Product received at distribution center",
    verified: true,
    transaction_hash: "0x1234567890abcdefj",
  },
];

export const mockProducts: Product[] = [
  mockProduct,
  {
    ...mockProduct,
    id: "PROD-002",
    name: "Premium Green Tea",
    description: "High-quality green tea from Japan",
    category: "Beverages",
    origin: {
      ...mockProduct.origin,
      location: "Kyoto, Japan",
      coordinates: { latitude: 35.0116, longitude: 135.7681 },
    },
    tags: ["green-tea", "premium", "japanese"],
    certifications: [
      {
        id: "CERT-002",
        name: "JAS Organic",
        issuer: "JAS",
        issued_at: 1640995200,
        expires_at: 1704067200,
        verified: true,
      },
    ],
  },
  {
    ...mockProduct,
    id: "PROD-003",
    name: "Wild Honey",
    description: "Pure wild honey from New Zealand",
    category: "Food",
    origin: {
      ...mockProduct.origin,
      location: "Christchurch, New Zealand",
      coordinates: { latitude: -43.5321, longitude: 172.6362 },
    },
    tags: ["honey", "wild", "new-zealand"],
    certifications: [
      {
        id: "CERT-003",
        name: "UMF Certified",
        issuer: "UMF",
        issued_at: 1640995200,
        expires_at: 1704067200,
        verified: true,
      },
    ],
  },
];

export const mockWallet = {
  address: "GBX2NJBSK3LUMRXF24F6EBV4TQJQU5WIK6HM4YJOTGZLJLOK5ZU6XHSY",
  publicKey: "GBX2NJBSK3LUMRXF24F6EBV4TQJQU5WIK6HM4YJOTGZLJLOK5ZU6XHSY",
  network: "testnet" as const,
  connected: true,
  balance: 1000.5,
};

export const mockBlockchainConfig = {
  network: "testnet" as const,
  rpcUrl: "https://soroban-testnet.stellar.org",
  contractAddress: "CBUWSKT2UGOAXK4ZREVDJV5XHSYB42PZ3CERU2ZFUTUMAZLJEHNZIECA",
  nativeToken: "XLM",
  explorerUrl: "https://stellar.expert",
  confirmationBlocks: 1,
};

export const createMockProduct = (overrides: Partial<Product> = {}): Product => ({
  ...mockProduct,
  id: `PROD-${Math.random().toString(36).substr(2, 9).toUpperCase()}`,
  ...overrides,
});

export const createMockTrackingEvent = (overrides: Partial<TrackingEvent> = {}): TrackingEvent => ({
  ...mockTrackingEvents[0],
  id: `EVENT-${Math.random().toString(36).substr(2, 9).toUpperCase()}`,
  timestamp: Math.floor(Date.now() / 1000),
  ...overrides,
});

export const createMockProductList = (count: number): Product[] => 
  Array.from({ length: count }, (_, i) => createMockProduct({
    id: `PROD-${String(i + 1).padStart(3, '0')}`,
    name: `Product ${i + 1}`,
    description: `Description for product ${i + 1}`,
  }));

export const createMockTrackingEventList = (count: number, productId: string): TrackingEvent[] =>
  Array.from({ length: count }, (_, i) => createMockTrackingEvent({
    id: `EVENT-${String(i + 1).padStart(3, '0')}`,
    product_id: productId,
    timestamp: Math.floor(Date.now() / 1000) + (i * 3600), // 1 hour apart
    note: `Event ${i + 1} for product ${productId}`,
  }));
