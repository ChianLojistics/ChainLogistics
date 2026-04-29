"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { useWallet } from "@/lib/hooks/useWallet";
import { getProductsByOwner } from "@/lib/contract/products";
import type { Product } from "@/lib/types/product";
import { ProductList } from "@/components/products/ProductList";
import { ProductFilters, type FilterState } from "@/components/products/ProductFilters";

export default function ProductsPage() {
  const { publicKey } = useWallet();
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const [products, setProducts] = useState<Product[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<FilterState>({
    search: "",
    owner: "",
    category: "",
    status: "all",
    dateFrom: "",
    dateTo: "",
  });

  useEffect(() => {
    const next: FilterState = {
      search: searchParams.get("q") ?? "",
      owner: searchParams.get("owner") ?? "",
      category: searchParams.get("category") ?? "",
      status: (searchParams.get("status") as FilterState["status"]) ?? "all",
      dateFrom: searchParams.get("from") ?? "",
      dateTo: searchParams.get("to") ?? "",
    };

    if (next.status !== "all" && next.status !== "active" && next.status !== "inactive") {
      next.status = "all";
    }

    setFilters(next);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const t = setTimeout(() => {
      const params = new URLSearchParams();
      if (filters.search.trim()) params.set("q", filters.search.trim());
      if (filters.owner) params.set("owner", filters.owner);
      if (filters.category) params.set("category", filters.category);
      if (filters.status !== "all") params.set("status", filters.status);
      if (filters.dateFrom) params.set("from", filters.dateFrom);
      if (filters.dateTo) params.set("to", filters.dateTo);

      const qs = params.toString();
      router.replace(qs ? `${pathname}?${qs}` : pathname);
    }, 250);

    return () => clearTimeout(t);
  }, [filters, pathname, router]);

  const fetchProducts = useCallback(async () => {
    if (!publicKey) {
      setProducts([]);
      setIsLoading(false);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const fetchedProducts = await getProductsByOwner(publicKey);
      setProducts(fetchedProducts);
    } catch (err) {
      console.error("Error fetching products:", err);
      setError(
        err instanceof Error
          ? err.message
          : "Failed to fetch products. Please check your connection and try again."
      );
    } finally {
      setIsLoading(false);
    }
  }, [publicKey]);

  // Fetch products
  useEffect(() => {
    fetchProducts();
  }, [fetchProducts]);

  // Extract unique categories and owners for filter dropdowns
  const availableCategories = useMemo(() => {
    const categories = new Set(products.map((p) => p.category));
    return Array.from(categories).sort();
  }, [products]);

  const availableOwners = useMemo(() => {
    const owners = new Set(products.map((p) => p.owner));
    return Array.from(owners).sort();
  }, [products]);

  const suggestions = useMemo(() => {
    const map = new Map<string, { id: string; label: string }>();
    for (const p of products) {
      map.set(`name:${p.id}`, { id: `name:${p.id}`, label: p.name });
      map.set(`id:${p.id}`, { id: `id:${p.id}`, label: p.id });
      const origin = p.origin?.location;
      if (origin) map.set(`origin:${origin}`, { id: `origin:${origin}`, label: origin });
    }
    return Array.from(map.values());
  }, [products]);

  return (
    <main className="mx-auto max-w-7xl px-6 py-10">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-zinc-900 mb-2">Products</h1>
        <p className="text-zinc-600">
          Search and filter registered products on the blockchain
        </p>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
          <p className="text-red-800 text-sm font-medium">Unable to load products</p>
          <p className="text-red-700 text-sm mt-1">{error}</p>
          <div className="mt-3">
            <button
              type="button"
              onClick={fetchProducts}
              disabled={isLoading}
              className="px-4 py-2 rounded-lg bg-red-600 text-white text-sm font-semibold hover:bg-red-700 disabled:opacity-50"
            >
              Retry
            </button>
          </div>
        </div>
      )}

      {!publicKey && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
          <p className="text-blue-800 text-sm">
            Please connect your wallet to view your products.
          </p>
        </div>
      )}

      <ProductFilters
        filters={filters}
        onFiltersChange={setFilters}
        availableCategories={availableCategories}
        availableOwners={availableOwners}
        suggestions={suggestions}
      />

      <ProductList products={products} filters={filters} isLoading={isLoading} />
    </main>
  );
}
