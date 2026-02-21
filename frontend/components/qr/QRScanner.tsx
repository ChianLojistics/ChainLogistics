"use client";

import React, { useEffect, useState } from 'react';
import { Html5QrcodeScanner, Html5QrcodeSupportedFormats } from 'html5-qrcode';
import { useRouter } from 'next/navigation';

interface QRScannerProps {
    onScanSuccess?: (decodedText: string) => void;
    onScanFailure?: (error: string) => void;
}

export default function QRScanner({ onScanSuccess, onScanFailure }: QRScannerProps) {
    const [error, setError] = useState<string | null>(null);
    const router = useRouter();

    useEffect(() => {
        // Determine if we are rendering on client-side
        if (typeof window === 'undefined') return;

        // Use html5-qrcode's scanner
        const scannerId = "qrcode-reader";
        const scanner = new Html5QrcodeScanner(
            scannerId,
            {
                fps: 10,
                qrbox: { width: 250, height: 250 },
                formatsToSupport: [Html5QrcodeSupportedFormats.QR_CODE],
                aspectRatio: 1.0,
            },
      /* verbose= */ false
        );

        const defaultSuccess = (decodedText: string) => {
            // Upon successful scan, we attempt to navigate directly if it's a URL
            scanner.clear().catch(console.error);

            try {
                const url = new URL(decodedText);
                if (url.pathname.includes('/verify/')) {
                    router.push(url.pathname);
                } else {
                    setError('Invalid ChainLogistics QR Code scanned.');
                }
            } catch {
                if (onScanSuccess) {
                    onScanSuccess(decodedText);
                } else {
                    setError(`Invalid QR content: ${decodedText}`);
                }
            }
        };

        scanner.render(
            (text) => {
                if (onScanSuccess) {
                    onScanSuccess(text);
                } else {
                    defaultSuccess(text);
                }
            },
            (errorMessage) => {
                if (onScanFailure) onScanFailure(errorMessage);
            }
        );

        return () => {
            scanner.clear().catch(console.error);
        };
    }, [onScanSuccess, onScanFailure, router]);

    return (
        <div className="w-full max-w-lg mx-auto bg-white p-6 rounded-2xl shadow">
            <h3 className="text-xl font-bold text-gray-800 mb-4 text-center">Scan Product QR Code</h3>
            {error && (
                <div className="mb-4 p-4 bg-red-50 text-red-700 border border-red-200 rounded-xl text-sm font-medium">
                    {error}
                </div>
            )}
            <div
                id="qrcode-reader"
                className="w-full overflow-hidden rounded-xl border border-gray-200"
            ></div>
            <p className="mt-4 text-sm text-gray-500 text-center">
                Point your camera at the QR code to verify product authenticity and track its supply chain history.
            </p>
        </div>
    );
}
