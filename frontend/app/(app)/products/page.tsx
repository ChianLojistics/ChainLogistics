"use client";

import { useState, useEffect, useMemo } from "react";
import { useWallet } from "@/lib/hooks/useWallet";
import { getProductsByOwner } from "@/lib/contract/products";
import type { Product } from "@/lib/types/product";
import { ProductList } from "@/components/products/ProductList";
import { ProductFilters, type FilterState } from "@/components/products/ProductFilters";

export default function ProductsPage() {
  const { publicKey } = useWallet();
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

  // Fetch products
  useEffect(() => {
    async function fetchProducts() {
      setIsLoading(true);
      setError(null);

      try {
        // For now, if no wallet is connected, we'll fetch from a default owner
        // In production, you might want to fetch from multiple owners or use an indexer
        const owner = publicKey || "GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";
        
        // TODO: In production, fetch from multiple owners or use an indexer
        // For now, we'll fetch from the connected wallet's address
        const fetchedProducts = await getProductsByOwner(owner);
        
        // If we have a connected wallet, only show their products by default
        // Otherwise, show all products (when indexer is implemented)
        setProducts(fetchedProducts);
      } catch (err) {
        console.error("Error fetching products:", err);
        setError(err instanceof Error ? err.message : "Failed to fetch products");
      } finally {
        setIsLoading(false);
      }
    }

    fetchProducts();
  }, [publicKey]);

  // Extract unique categories and owners for filter dropdowns
  const availableCategories = useMemo(() => {
    const categories = new Set(products.map((p) => p.category));
    return Array.from(categories).sort();
  }, [products]);

  const availableOwners = useMemo(() => {
    const owners = new Set(products.map((p) => p.owner));
    return Array.from(owners).sort();
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
          <p className="text-red-800 text-sm">{error}</p>
        </div>
      )}

      {!publicKey && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
          <p className="text-blue-800 text-sm">
            Connect your wallet to view your products, or browse all products
            below.
          </p>
        </div>
      )}

      <ProductFilters
        filters={filters}
        onFiltersChange={setFilters}
        availableCategories={availableCategories}
        availableOwners={availableOwners}
      />

      <ProductList products={products} filters={filters} isLoading={isLoading} />
    </main>
  );
}
