import React from 'react';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { MapPin, Package, CalendarPlus, ChevronRight } from 'lucide-react';
import Link from 'next/link';

export interface Product {
    id: string;
    name: string;
    origin: string;
    status: 'active' | 'inactive';
    eventCount: number;
}

interface ProductCardProps {
    product: Product;
}

export function ProductCard({ product }: ProductCardProps) {
    return (
        <Card className="group relative overflow-hidden transition-all hover:shadow-lg">
            <Link href={`/products/${product.id}`} className="absolute inset-0 z-0">
                <span className="sr-only">View Details for {product.name}</span>
            </Link>

            <CardHeader className="pb-4">
                <div className="flex items-start justify-between">
                    <div className="space-y-1">
                        <CardTitle className="flex items-center gap-2 text-xl">
                            <Package className="h-5 w-5 text-muted-foreground" />
                            {product.name}
                        </CardTitle>
                        <CardDescription className="text-sm">ID: {product.id}</CardDescription>
                    </div>
                    <Badge
                        variant={product.status === 'active' ? 'default' : 'secondary'}
                        className={product.status === 'active' ? 'bg-green-500 hover:bg-green-600' : ''}
                    >
                        {product.status.charAt(0).toUpperCase() + product.status.slice(1)}
                    </Badge>
                </div>
            </CardHeader>

            <CardContent className="pb-4">
                <div className="flex flex-col space-y-2 text-sm text-muted-foreground">
                    <div className="flex items-center gap-2">
                        <MapPin className="h-4 w-4" />
                        <span>Origin: {product.origin}</span>
                    </div>
                    <div className="flex items-center gap-2">
                        <Badge variant="outline" className="font-normal">
                            {product.eventCount} {product.eventCount === 1 ? 'Event' : 'Events'}
                        </Badge>
                    </div>
                </div>
            </CardContent>

            <CardFooter className="flex items-center justify-end gap-2 z-10 relative">
                <Button variant="outline" size="sm" asChild>
                    <Link href={`/products/${product.id}/add-event`}>
                        <CalendarPlus className="mr-2 h-4 w-4" />
                        Add Event
                    </Link>
                </Button>
                <Button variant="secondary" size="sm" asChild>
                    <Link href={`/products/${product.id}`}>
                        <ChevronRight className="mr-2 h-4 w-4" />
                        View
                    </Link>
                </Button>
            </CardFooter>
        </Card>
    );
}
