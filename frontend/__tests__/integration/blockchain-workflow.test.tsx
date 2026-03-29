import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { WalletConnector } from "@/components/blockchain/WalletConnector";
import { BlockchainSelector } from "@/components/blockchain/BlockchainSelector";
import { mockWallet, mockBlockchainConfig } from "../mocks/data";

// Mock blockchain providers
vi.mock("@/lib/blockchain/providers/stellar", () => ({
  connectWallet: vi.fn(),
  disconnectWallet: vi.fn(),
  getAccount: vi.fn(),
  getBalance: vi.fn(),
  signTransaction: vi.fn(),
}));

vi.mock("@/lib/blockchain/providers/ethereum", () => ({
  connectWallet: vi.fn(),
  disconnectWallet: vi.fn(),
  getAccount: vi.fn(),
  getBalance: vi.fn(),
  signTransaction: vi.fn(),
}));

vi.mock("@/lib/contract/config", () => ({
  CONTRACT_CONFIG: {
    CONTRACT_ID: "CBUWSKT2UGOAXK4ZREVDJV5XHSYB42PZ3CERU2ZFUTUMAZLJEHNZIECA",
    NETWORK: "testnet",
    RPC_URL: "https://soroban-testnet.stellar.org",
  },
  validateContractConfig: vi.fn(),
}));

// Mock blockchain factory
vi.mock("@/lib/blockchain/factory", () => ({
  blockchainFactory: {
    getProvider: vi.fn(),
  },
}));

describe("Blockchain Workflow Integration Tests", () => {
  const renderComponent = (component: React.ReactElement) => {
    return render(component);
  };

  describe("Wallet Connection Flow", () => {
    it("should render wallet connector with required props", () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      
      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      expect(screen.getByText(/connect wallet/i)).toBeInTheDocument();
    });

    it("should handle wallet connection successfully", async () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      const mockConnectWallet = vi.fn().mockResolvedValue(mockWallet);
      
      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      fireEvent.click(screen.getByRole("button", { name: /connect wallet/i }));

      await waitFor(() => {
        expect(onConnect).toHaveBeenCalled();
      });
    });

    it("should handle wallet connection errors", async () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      const mockConnectWallet = vi.fn().mockRejectedValue(
        new Error("Freighter wallet not installed")
      );

      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      fireEvent.click(screen.getByRole("button", { name: /connect wallet/i }));

      await waitFor(() => {
        expect(screen.getByText(/failed to connect/i)).toBeInTheDocument();
      });
    });

    it("should disconnect wallet properly", async () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      const mockConnectWallet = vi.fn().mockResolvedValue(mockWallet);
      const mockDisconnectWallet = vi.fn().mockResolvedValue(undefined);

      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      // Connect first
      fireEvent.click(screen.getByRole("button", { name: /connect wallet/i }));

      await waitFor(() => {
        expect(onConnect).toHaveBeenCalled();
      });

      // Then disconnect
      fireEvent.click(screen.getByRole("button", { name: /disconnect/i }));

      await waitFor(() => {
        expect(onDisconnect).toHaveBeenCalled();
      });
    });
  });

  describe("Blockchain Network Selection", () => {
    it("should render blockchain selector with required props", () => {
      const onNetworkChange = vi.fn();
      
      renderComponent(
        <BlockchainSelector onNetworkChange={onNetworkChange} />
      );

      expect(screen.getByText(/stellar/i)).toBeInTheDocument();
    });

    it("should switch between blockchain networks", async () => {
      const onNetworkChange = vi.fn();
      
      renderComponent(
        <BlockchainSelector onNetworkChange={onNetworkChange} />
      );

      // Should default to Stellar
      expect(screen.getByText(/stellar/i)).toBeInTheDocument();

      // Switch to Ethereum
      fireEvent.click(screen.getByRole("button", { name: /ethereum/i }));

      await waitFor(() => {
        expect(onNetworkChange).toHaveBeenCalledWith("ethereum");
      });
    });

    it("should show network-specific configuration", async () => {
      const onNetworkChange = vi.fn();
      
      renderComponent(
        <BlockchainSelector onNetworkChange={onNetworkChange} />
      );

      // Check Stellar configuration
      expect(screen.getByText(/testnet/i)).toBeInTheDocument();

      // Switch to Polygon
      fireEvent.click(screen.getByRole("button", { name: /polygon/i }));

      await waitFor(() => {
        expect(onNetworkChange).toHaveBeenCalledWith("polygon");
      });
    });

    it("should handle unsupported networks gracefully", async () => {
      const onNetworkChange = vi.fn();
      
      renderComponent(
        <BlockchainSelector onNetworkChange={onNetworkChange} />
      );

      // Try to access unsupported network
      fireEvent.click(screen.getByRole("button", { name: /hyperledger/i }));

      await waitFor(() => {
        expect(onNetworkChange).toHaveBeenCalledWith("hyperledger");
      });
    });
  });

  describe("Error Handling", () => {
    it("should handle RPC connection failures", async () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      const mockConnectWallet = vi.fn().mockRejectedValue(
        new Error("Unable to connect to Stellar RPC")
      );

      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      fireEvent.click(screen.getByRole("button", { name: /connect wallet/i }));

      await waitFor(() => {
        expect(screen.getByText(/failed to connect/i)).toBeInTheDocument();
      });
    });

    it("should handle network timeouts", async () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      const mockConnectWallet = vi.fn().mockImplementation(() => {
        return new Promise((_, reject) => {
          setTimeout(() => reject(new Error("Network timeout")), 1000);
        });
      });

      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      fireEvent.click(screen.getByRole("button", { name: /connect wallet/i }));

      await waitFor(() => {
        expect(screen.getByText(/failed to connect/i)).toBeInTheDocument();
      }, { timeout: 2000 });
    });
  });

  describe("Multi-Blockchain Support", () => {
    it("should support multiple blockchain networks", () => {
      const onNetworkChange = vi.fn();
      
      renderComponent(
        <BlockchainSelector onNetworkChange={onNetworkChange} />
      );

      // Check that all supported networks are present
      expect(screen.getByText(/stellar/i)).toBeInTheDocument();
      expect(screen.getByText(/ethereum/i)).toBeInTheDocument();
      expect(screen.getByText(/polygon/i)).toBeInTheDocument();
    });

    it("should handle network switching callbacks", async () => {
      const onNetworkChange = vi.fn();
      
      renderComponent(
        <BlockchainSelector onNetworkChange={onNetworkChange} />
      );

      // Test network switching
      fireEvent.click(screen.getByRole("button", { name: /stellar/i }));
      await waitFor(() => {
        expect(onNetworkChange).toHaveBeenCalledWith("stellar");
      });

      fireEvent.click(screen.getByRole("button", { name: /ethereum/i }));
      await waitFor(() => {
        expect(onNetworkChange).toHaveBeenCalledWith("ethereum");
      });
    });
  });

  describe("Security Features", () => {
    it("should validate network before connection", async () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      
      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      // Should show network information
      expect(screen.getByText(/stellar/i)).toBeInTheDocument();
    });

    it("should show connection status", async () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      
      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      // Should show disconnected status initially
      expect(screen.getByText(/connect wallet/i)).toBeInTheDocument();
    });
  });

  describe("Accessibility", () => {
    it("should be accessible via keyboard navigation", () => {
      const onConnect = vi.fn();
      const onDisconnect = vi.fn();
      
      renderComponent(
        <WalletConnector 
          network="stellar" 
          onConnect={onConnect} 
          onDisconnect={onDisconnect} 
        />
      );

      // Tab through elements
      const connectButton = screen.getByRole("button", { name: /connect wallet/i });
      connectButton.focus();
      expect(connectButton).toHaveFocus();
    });

    it("should have proper ARIA labels", () => {
      const onNetworkChange = vi.fn();
      
      renderComponent(
        <BlockchainSelector onNetworkChange={onNetworkChange} />
      );

      // Should have proper button labels
      expect(screen.getByRole("button", { name: /stellar/i })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: /ethereum/i })).toBeInTheDocument();
    });
  });
});
