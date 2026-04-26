import dynamic from "next/dynamic";
import { LoadingSpinner } from "@/components/ui/loading-spinner";

// Code-split: ProductRegistrationForm pulls in react-hook-form + zod + contract
// clients — not needed on initial paint, so defer it to its own chunk.
const ProductRegistrationForm = dynamic(
  () =>
    import("@/components/forms/ProductRegistrationForm").then((m) => ({
      default: m.ProductRegistrationForm,
    })),
  {
    loading: () => (
      <div className="flex items-center justify-center py-24">
        <LoadingSpinner />
      </div>
    ),
  }
);

export default function RegisterProductPage() {
  return (
    <main className="mx-auto max-w-4xl px-4 py-12">
      <div className="text-center mb-10">
        <h1 className="text-3xl font-bold text-zinc-900">Product Registration</h1>
        <p className="text-zinc-600 mt-2">Registers your product assets on the Stellar blockchain for verified tracking.</p>
      </div>

      <ProductRegistrationForm />
    </main>
  );
}
