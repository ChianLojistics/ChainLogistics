"use client";

import React from 'react';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { MapPin, Package, CalendarPlus, ChevronRight, Tag, User } from 'lucide-react';
import Link from 'next/link';
import type { Product } from "@/lib/types/product";
import { normalizeUnixSeconds } from "@/lib/types/product";
import { shortenPublicKey } from "@/lib/utils/format";

interface ProductCardProps {
  product: Product;
}

export function ProductCard({ product }: ProductCardProps) {
  const formatDate = (timestamp: number) => {
    const seconds = normalizeUnixSeconds(timestamp);
    return new Date(seconds * 1000).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  };

  const owner = product.owner || product.owner_address || "";
  const isActive = product.active ?? product.is_active ?? product.isActive ?? false;

  return (
    <Card className="block group relative overflow-hidden bg-white hover:border-zinc-300 transition-all duration-200 hover:shadow-lg">
      <Link href={`/products/${product.id}`} className="absolute inset-0 z-0">
        <span className="sr-only">View Details for {product.name}</span>
      </Link>

      <CardHeader className="pb-4">
        <div className="flex items-start justify-between">
          <div className="space-y-1">
            <CardTitle className="flex items-center gap-2 text-xl">
              <Package className="h-5 w-5 text-muted-foreground" aria-hidden="true" />
              {product.name}
            </CardTitle>
            <CardDescription className="text-sm font-mono">ID: {product.id}</CardDescription>
          </div>
          <Badge
            variant={isActive ? 'default' : 'secondary'}
            className={isActive ? 'bg-green-100 text-green-700 hover:bg-green-200' : 'bg-red-100 text-red-700 hover:bg-red-200'}
          >
            {isActive ? 'Active' : 'Inactive'}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="pb-4">
        <div className="flex flex-col space-y-2 text-sm text-zinc-600">
          <div className="flex items-center gap-2">
            <MapPin className="h-4 w-4" aria-hidden="true" />
            <span className="font-medium">Origin:</span>
            <span>{product.origin?.location || product.origin_location || "N/A"}</span>
          </div>
          <div className="flex items-center gap-2">
            <Tag className="h-4 w-4" aria-hidden="true" />
            <span className="font-medium">Category:</span>
            <span>{product.category}</span>
          </div>
          <div className="flex items-center gap-2">
            <User className="h-4 w-4" aria-hidden="true" />
            <span className="font-medium">Owner:</span>
            <span className="font-mono">{shortenPublicKey(owner)}</span>
          </div>
          <div className="flex items-center gap-2">
            <CalendarPlus className="h-4 w-4" aria-hidden="true" />
            <span className="font-medium">Created:</span>
            <span>{formatDate(product.created_at ?? product.createdAt ?? 0)}</span>
          </div>
        </div>
      </CardContent>

      <CardFooter className="flex items-center justify-between gap-2 z-10 relative border-t border-zinc-100 pt-4">
        <div className="flex items-center gap-2 text-sm text-zinc-600">
          <Badge variant="outline" className="font-normal bg-zinc-50 border-zinc-200">
            {product.eventCount ?? 0} {product.eventCount === 1 ? 'Event' : 'Events'}
          </Badge>
        </div>

        <div className="flex items-center gap-2">
          {product.tags && product.tags.length > 0 && (
            <div className="hidden sm:flex gap-1 flex-wrap mr-2">
              {product.tags.slice(0, 2).map((tag, idx) => (
                <span key={idx} className="px-2 py-0.5 bg-zinc-100 text-zinc-600 text-xs rounded">
                  {tag}
                </span>
              ))}
            </div>
          )}
          <Button variant="outline" size="sm" asChild>
            <Link href={`/products/${product.id}/add-event`}>
              <CalendarPlus className="sm:mr-2 h-4 w-4" />
              <span className="hidden sm:inline">Add Event</span>
            </Link>
          </Button>
          <Button variant="secondary" size="sm" asChild>
            <Link href={`/products/${product.id}`}>
              <span className="hidden sm:inline">View</span>
              <ChevronRight className="sm:ml-2 h-4 w-4" />
            </Link>
          </Button>
        </div>
      </CardFooter>
    </Card>
  );
}
