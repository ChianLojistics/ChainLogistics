'use client';

import React, { useEffect, useState } from 'react';
import { WebSocketClient } from '@/lib/websocket/client';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface RealtimeEvent {
  id: string;
  product_id: string;
  event_type: string;
  location: string;
  timestamp: number;
  data: Record<string, unknown>;
}

interface RealtimeEventListenerProps {
  productId?: string;
}

export function RealtimeEventListener({ productId }: RealtimeEventListenerProps) {
  const [events, setEvents] = useState<RealtimeEvent[]>([]);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const wsUrl = `${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001'}/ws`;
    const client = new WebSocketClient(wsUrl);

    client.connect().then(() => {
      setConnected(true);

      // Subscribe to product events
      const channel = productId ? `product:${productId}` : 'events:all';
      client.subscribe(channel, (data: Record<string, unknown>) => {
        setEvents((prev) => [data as unknown as RealtimeEvent, ...prev].slice(0, 50)); // Keep last 50 events
      });
    });

    return () => {
      client.disconnect();
    };
  }, [productId]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Real-time Events</h2>
        <Badge variant={connected ? 'default' : 'secondary'}>
          {connected ? 'Connected' : 'Disconnected'}
        </Badge>
      </div>

      {events.length === 0 ? (
        <p className="text-gray-500">Waiting for events...</p>
      ) : (
        <div className="space-y-2">
          {events.map((event) => (
            <Card key={event.id} className="p-4">
              <div className="flex justify-between items-start">
                <div>
                  <p className="font-semibold">{event.event_type}</p>
                  <p className="text-sm text-gray-600">{event.location}</p>
                  <p className="text-xs text-gray-500 mt-1">
                    {new Date(event.timestamp).toLocaleString()}
                  </p>
                </div>
                <Badge variant="outline">{event.product_id.slice(0, 8)}</Badge>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
