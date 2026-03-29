import { PRODUCT_REGISTRATION_DELAY_MS } from "@/lib/constants";

export type ProductData = {
    id: string;
    name: string;
    origin: string;
    description?: string;
    category: string;
};

export async function registerProductOnChain(
    publicKey: string,
    data: ProductData
): Promise<string> {
    // In a real Soroban contract interaction, we would use the Stellar SDK
    // to invoke a contract method. For now, we'll implement the logic
    // to build a transaction that could be used for this.

    // This is a placeholder for actual Soroban contract invocation.
    // In a real scenario, you'd use `new Contract(CONTRACT_CONFIG.CONTRACT_ID).call("register", ...)`

    // Validate contract configuration
    if (!publicKey || !data.id) {
        throw new Error('Invalid contract parameters');
    }

    // Simulate a delay
    await new Promise(resolve => setTimeout(resolve, PRODUCT_REGISTRATION_DELAY_MS));

    // Return a mock transaction hash
    return "t_" + Math.random().toString(36).substring(2, 15);
}
