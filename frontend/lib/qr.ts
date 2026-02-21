import QRCode from 'qrcode';

export async function generateProductQR(productId: string): Promise<string> {
    const url = getVerificationUrl(productId);
    return QRCode.toDataURL(url, {
        width: 300,
        margin: 2,
        color: {
            dark: '#000000',
            light: '#FFFFFF'
        }
    });
}

export async function generateProductQRSVG(productId: string): Promise<string> {
    const url = getVerificationUrl(productId);
    return QRCode.toString(url, {
        type: 'svg',
        width: 300,
        margin: 2,
        color: {
            dark: '#000000',
            light: '#FFFFFF'
        }
    });
}

export function getVerificationUrl(productId: string): string {
    const baseUrl = process.env.NEXT_PUBLIC_APP_URL || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost:3000');
    return `${baseUrl}/verify/${productId}`;
}
