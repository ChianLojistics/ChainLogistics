"use client";

import Link from "next/link";
import type { Product } from "@/lib/types/product";
import { shortenPublicKey } from "@/lib/utils/format";

type ProductCardProps = {
  product: Product;
};

export function ProductCard({ product }: ProductCardProps) {
  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  };

  return (
    <Link
      href={`/products/${product.id}`}
      className="block bg-white rounded-lg border border-zinc-200 p-6 hover:border-zinc-300 hover:shadow-md transition-all duration-200"
    >
      <div className="flex items-start justify-between mb-4">
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-zinc-900 mb-1">
            {product.name}
          </h3>
          <p className="text-sm text-zinc-500 font-mono">{product.id}</p>
        </div>
        <span
          className={`px-3 py-1 rounded-full text-xs font-medium ${
            product.active
              ? "bg-green-100 text-green-700"
              : "bg-red-100 text-red-700"
          }`}
        >
          {product.active ? "Active" : "Inactive"}
        </span>
      </div>

      <div className="space-y-2 mb-4">
        <div className="flex items-center text-sm text-zinc-600">
          <span className="font-medium mr-2">Origin:</span>
          <span>{product.origin.location}</span>
        </div>
        <div className="flex items-center text-sm text-zinc-600">
          <span className="font-medium mr-2">Category:</span>
          <span>{product.category}</span>
        </div>
        <div className="flex items-center text-sm text-zinc-600">
          <span className="font-medium mr-2">Owner:</span>
          <span className="font-mono">{shortenPublicKey(product.owner)}</span>
        </div>
        <div className="flex items-center text-sm text-zinc-600">
          <span className="font-medium mr-2">Created:</span>
          <span>{formatDate(product.created_at)}</span>
        </div>
      </div>

      <div className="flex items-center justify-between pt-4 border-t border-zinc-100">
        <div className="flex items-center gap-2">
          <svg
            className="w-4 h-4 text-zinc-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
            />
          </svg>
          <span className="text-sm text-zinc-600">
            {product.eventCount ?? 0} event{product.eventCount !== 1 ? "s" : ""}
          </span>
        </div>
        {product.tags.length > 0 && (
          <div className="flex gap-1 flex-wrap">
            {product.tags.slice(0, 2).map((tag, idx) => (
              <span
                key={idx}
                className="px-2 py-0.5 bg-zinc-100 text-zinc-600 text-xs rounded"
              >
                {tag}
              </span>
            ))}
            {product.tags.length > 2 && (
              <span className="px-2 py-0.5 bg-zinc-100 text-zinc-600 text-xs rounded">
                +{product.tags.length - 2}
              </span>
            )}
          </div>
        )}
      </div>
    </Link>
  );
}
