'use client';

import React, { useState } from 'react';
import { blockchainFactory } from '@/lib/blockchain/factory';
import { BlockchainNetwork, WalletConnection } from '@/lib/blockchain/types';
import { Button } from '@/components/ui/button';
import { toast } from 'sonner';

interface WalletConnectorProps {
  network: BlockchainNetwork;
  onConnect: (connection: WalletConnection) => void;
  onDisconnect: () => void;
}

export function WalletConnector({ network, onConnect, onDisconnect }: WalletConnectorProps) {
  const [isConnecting, setIsConnecting] = useState(false);
  const [connected, setConnected] = useState(false);
  const [address, setAddress] = useState<string>('');

  const handleConnect = async () => {
    setIsConnecting(true);
    try {
      const provider = blockchainFactory.getProvider(network);
      const connection = await provider.connect();
      setConnected(true);
      setAddress(connection.address);
      onConnect(connection);
      toast.success(`Connected to ${network} wallet`);
    } catch (error) {
      toast.error(`Failed to connect: ${error}`);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    try {
      const provider = blockchainFactory.getProvider(network);
      await provider.disconnect();
      setConnected(false);
      setAddress('');
      onDisconnect();
      toast.success('Wallet disconnected');
    } catch (error) {
      toast.error(`Failed to disconnect: ${error}`);
    }
  };

  if (connected) {
    return (
      <div className="flex items-center gap-4">
        <div className="text-sm">
          <p className="text-gray-600">Connected Address:</p>
          <p className="font-mono text-sm break-all">{address}</p>
        </div>
        <Button variant="destructive" onClick={handleDisconnect}>
          Disconnect
        </Button>
      </div>
    );
  }

  return (
    <Button onClick={handleConnect} disabled={isConnecting}>
      {isConnecting ? 'Connecting...' : 'Connect Wallet'}
    </Button>
  );
}
