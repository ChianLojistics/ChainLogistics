export default async function ProductDetailsPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;

  return (
    <main className="mx-auto max-w-3xl px-6 py-10">
      <h1 className="text-2xl font-semibold">Product: {id}</h1>
      <p className="text-zinc-600">Events will appear here.</p>
    </main>
  );
}
