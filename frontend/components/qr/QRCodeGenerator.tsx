"use client";

/* eslint-disable @next/next/no-img-element */
import React, { useState, useEffect, useRef } from 'react';
import { generateProductQR, generateProductQRSVG, getVerificationUrl } from '@/lib/qr';

interface QRCodeGeneratorProps {
    productId: string;
}

export default function QRCodeGenerator({ productId }: QRCodeGeneratorProps) {
    const [qrCodeUrl, setQrCodeUrl] = useState<string>('');
    const [copied, setCopied] = useState(false);
    const printRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        generateProductQR(productId).then(setQrCodeUrl).catch(console.error);
    }, [productId]);

    const handleCopyLink = async () => {
        const url = getVerificationUrl(productId);
        try {
            await navigator.clipboard.writeText(url);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error('Failed to copy text: ', err);
        }
    };

    const handleDownloadPNG = () => {
        if (!qrCodeUrl) return;
        const a = document.createElement('a');
        a.href = qrCodeUrl;
        a.download = `product-${productId}-qr.png`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
    };

    const handleDownloadSVG = async () => {
        try {
            const svgString = await generateProductQRSVG(productId);
            const blob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `product-${productId}-qr.svg`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (error) {
            console.error('Failed to download SVG:', error);
        }
    };

    const handlePrint = () => {
        const printContent = printRef.current;
        if (!printContent) return;

        // Use a lightweight popup approach for printing
        const popupWin = window.open('', '_blank', 'width=600,height=600');
        if (popupWin) {
            popupWin.document.open();
            popupWin.document.write(`
        <html>
          <head>
            <title>Print QR Code</title>
            <style>
              body { font-family: system-ui, -apple-system, sans-serif; display: flex; justify-content: center; align-items: center; min-height: 100vh; margin: 0; background-color: #f9fafb; padding: 20px;}
              .print-container { text-align: center; border: 2px dashed #d1d5db; padding: 40px; border-radius: 12px; background: white; max-width: 400px; width: 100%; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1); }
              img { max-width: 100%; height: auto; margin: 20px auto; display: block; }
              h2 { margin-top: 0; color: #111827; font-size: 24px; }
              .id-text { color: #6b7280; font-family: monospace; font-size: 14px; background: #f3f4f6; padding: 6px 12px; border-radius: 6px; display: inline-block; margin-bottom: 20px; word-break: break-all; }
              .footer { color: #4b5563; font-size: 14px; font-weight: 500; }
            </style>
          </head>
          <body onload="window.print(); window.close();">
            <div class="print-container">
              <h2>Product Verification</h2>
              <div class="id-text">${productId}</div>
              ${printContent.innerHTML}
              <p class="footer">Scan to verify authenticity & track origin</p>
            </div>
          </body>
        </html>
      `);
            popupWin.document.close();
        }
    };

    return (
        <div className="flex flex-col items-center p-6 bg-white rounded-2xl shadow border border-gray-100 max-w-sm w-full">
            <h3 className="text-xl font-bold mb-6 text-gray-800">Product QR Code</h3>

            <div className="bg-gray-50 p-6 rounded-xl mb-6 shadow-inner border border-gray-200" ref={printRef}>
                {qrCodeUrl ? (
                    <img src={qrCodeUrl} alt={`QR Code for ${productId}`} className="w-56 h-56 object-contain block mx-auto mix-blend-multiply" />
                ) : (
                    <div className="w-56 h-56 flex items-center justify-center text-gray-400 font-medium">
                        <span className="animate-pulse">Generating...</span>
                    </div>
                )}
            </div>

            <div className="flex flex-col gap-3 w-full">
                <button
                    onClick={handleCopyLink}
                    className="flex items-center justify-center px-4 py-2.5 bg-indigo-50 text-indigo-700 font-medium rounded-xl hover:bg-indigo-100 transition-colors focus:ring-2 focus:ring-indigo-200"
                >
                    {copied ? 'Link Copied!' : 'Copy Verification Link'}
                </button>

                <div className="grid grid-cols-2 gap-3">
                    <button
                        onClick={handleDownloadPNG}
                        disabled={!qrCodeUrl}
                        className="flex items-center justify-center px-4 py-2 bg-gray-100 font-medium text-gray-700 rounded-xl hover:bg-gray-200 transition-colors disabled:opacity-50"
                    >
                        Download PNG
                    </button>
                    <button
                        onClick={handleDownloadSVG}
                        disabled={!qrCodeUrl}
                        className="flex items-center justify-center px-4 py-2 bg-gray-100 font-medium text-gray-700 rounded-xl hover:bg-gray-200 transition-colors disabled:opacity-50"
                    >
                        Download SVG
                    </button>
                </div>

                <button
                    onClick={handlePrint}
                    disabled={!qrCodeUrl}
                    className="flex items-center justify-center px-4 py-3 mt-2 bg-gray-900 font-medium text-white shadow-md rounded-xl hover:bg-gray-800 transition-colors disabled:opacity-50 focus:ring-2 focus:ring-gray-300 focus:ring-offset-2"
                >
                    Print Shipping Label
                </button>
            </div>
        </div>
    );
}
