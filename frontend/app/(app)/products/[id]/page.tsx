import { Timeline } from "@/components/tracking";

export default async function ProductDetailsPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;

  return (
    <main className="mx-auto max-w-4xl px-6 py-10">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">Product Tracking</h1>
        <p className="mt-2 text-sm text-gray-600">
          Product ID: <span className="font-mono font-medium">{id}</span>
        </p>
      </div>

      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
        <h2 className="mb-6 text-xl font-semibold text-gray-900">
          Supply Chain Timeline
        </h2>
        <Timeline productId={id} />
      </div>
    </main>
  );
}
