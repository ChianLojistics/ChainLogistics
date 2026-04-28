import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { WalletConnect } from './WalletConnect';

// Mock the wallet store
const mockUseWalletStore = vi.fn();
vi.mock('@/lib/state/wallet.store', () => ({
  useWalletStore: () => mockUseWalletStore(),
}));

// Mock sonner toast
const mockToastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: {
    success: (...args: unknown[]) => mockToastSuccess(...args),
  },
}));

// Mock navigator.clipboard
const mockClipboardWriteText = vi.fn();
Object.assign(navigator, {
  clipboard: {
    writeText: mockClipboardWriteText,
  },
});

describe('WalletConnect component', () => {
  const mockConnect = vi.fn();
  const mockDisconnect = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render connect button when disconnected', () => {
    mockUseWalletStore.mockReturnValue({
      status: 'disconnected',
      publicKey: null,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: null,
    });

    render(<WalletConnect />);

    const button = screen.getByRole('button', { name: /connect freighter wallet/i });
    expect(button).toBeInTheDocument();
    expect(button).not.toBeDisabled();
  });

  it('should show loading state when connecting', () => {
    mockUseWalletStore.mockReturnValue({
      status: 'connecting',
      publicKey: null,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: null,
    });

    render(<WalletConnect />);

    const button = screen.getByRole('button', { name: /connect freighter wallet/i });
    expect(button).toBeInTheDocument();
    expect(button).toHaveAttribute('aria-busy', 'true');
    expect(button).toBeDisabled();
  });

  it('should display truncated address when connected', () => {
    const publicKey = 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7';
    mockUseWalletStore.mockReturnValue({
      status: 'connected',
      publicKey,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: null,
    });

    render(<WalletConnect />);

    const addressButton = screen.getByRole('button', { name: /wallet connected/i });
    expect(addressButton).toBeInTheDocument();
    expect(addressButton).toHaveTextContent('GCFXHS...Z7Z7');
  });

  it('should call connect when button is clicked', async () => {
    mockUseWalletStore.mockReturnValue({
      status: 'disconnected',
      publicKey: null,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: null,
    });

    render(<WalletConnect />);

    const button = screen.getByRole('button', { name: /connect freighter wallet/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(mockConnect).toHaveBeenCalledTimes(1);
    });
  });

  it('should call disconnect when disconnect button is clicked', async () => {
    const publicKey = 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7';
    mockUseWalletStore.mockReturnValue({
      status: 'connected',
      publicKey,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: null,
    });

    render(<WalletConnect />);

    const disconnectButton = screen.getByRole('button', { name: /disconnect wallet/i });
    fireEvent.click(disconnectButton);

    await waitFor(() => {
      expect(mockDisconnect).toHaveBeenCalledTimes(1);
    });
  });

  it('should display error message when connection fails', () => {
    mockUseWalletStore.mockReturnValue({
      status: 'error',
      publicKey: null,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: 'User rejected request',
    });

    render(<WalletConnect />);

    const errorAlert = screen.getByRole('alert');
    expect(errorAlert).toBeInTheDocument();
    expect(errorAlert).toHaveTextContent('User rejected request');
  });

  it('should show Freighter install link when wallet not found error', () => {
    mockUseWalletStore.mockReturnValue({
      status: 'error',
      publicKey: null,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: 'Freighter wallet not installed',
    });

    render(<WalletConnect />);

    const installLink = screen.getByText(/install here/i);
    expect(installLink).toBeInTheDocument();
    expect(installLink).toHaveAttribute('href', 'https://www.freighter.app/');
    expect(installLink).toHaveAttribute('target', '_blank');
    expect(installLink).toHaveAttribute('rel', 'noreferrer');
  });

  it('should copy address to clipboard when clicked', async () => {
    const publicKey = 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7';

    mockUseWalletStore.mockReturnValue({
      status: 'connected',
      publicKey,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: null,
    });

    render(<WalletConnect />);

    const addressButton = screen.getByRole('button', { name: /wallet connected/i });
    fireEvent.click(addressButton);

    await waitFor(() => {
      expect(mockClipboardWriteText).toHaveBeenCalledWith(publicKey);
      expect(mockToastSuccess).toHaveBeenCalledWith('Address copied to clipboard');
    });
  });

  it('should have correct accessibility attributes', () => {
    mockUseWalletStore.mockReturnValue({
      status: 'disconnected',
      publicKey: null,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: null,
    });

    render(<WalletConnect />);

    const button = screen.getByRole('button', { name: /connect freighter wallet/i });
    expect(button).toHaveAttribute('aria-label', 'Connect Freighter wallet');
  });

  it('should have accessible group when connected', () => {
    const publicKey = 'GCFXHS5DRCQZ4QZ7Z4X7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7Z7';
    mockUseWalletStore.mockReturnValue({
      status: 'connected',
      publicKey,
      connect: mockConnect,
      disconnect: mockDisconnect,
      error: null,
    });

    render(<WalletConnect />);

    const group = screen.getByRole('group', { name: /wallet controls/i });
    expect(group).toBeInTheDocument();
  });
});
