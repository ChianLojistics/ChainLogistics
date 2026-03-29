'use client';

import React, { useState } from 'react';
import { getSupportedNetworks, getNetworkName } from '@/lib/blockchain/config';
import { BlockchainNetwork } from '@/lib/blockchain/types';
import { Button } from '@/components/ui/button';

interface BlockchainSelectorProps {
  onNetworkChange: (network: BlockchainNetwork) => void;
  selectedNetwork?: BlockchainNetwork;
}

export function BlockchainSelector({ onNetworkChange, selectedNetwork }: BlockchainSelectorProps) {
  const networks = getSupportedNetworks();
  const [selected, setSelected] = useState<BlockchainNetwork>(selectedNetwork || 'stellar');

  const handleChange = (network: BlockchainNetwork) => {
    setSelected(network);
    onNetworkChange(network);
  };

  return (
    <div className="flex flex-col gap-4">
      <label className="text-sm font-medium">Select Blockchain Network</label>
      <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
        {networks.map((network) => (
          <Button
            key={network}
            variant={selected === network ? 'default' : 'outline'}
            onClick={() => handleChange(network)}
            className="w-full"
          >
            {getNetworkName(network)}
          </Button>
        ))}
      </div>
    </div>
  );
}
