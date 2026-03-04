import Link from "next/link";

export default function NotFound() {
  return (
    <div className="min-h-screen bg-zinc-50">
      <div className="mx-auto flex min-h-screen max-w-2xl flex-col items-center justify-center px-6 py-16 text-center">
        <h1 className="text-3xl font-bold text-zinc-900">Page not found</h1>
        <p className="mt-2 text-sm text-zinc-600">
          The page you’re looking for doesn’t exist or may have been moved.
        </p>
        <div className="mt-6 flex items-center justify-center gap-3">
          <Link
            href="/"
            className="rounded-lg bg-zinc-900 px-4 py-2 text-sm font-semibold text-white hover:bg-zinc-800"
          >
            Go home
          </Link>
          <Link
            href="/products"
            className="rounded-lg border border-zinc-300 bg-white px-4 py-2 text-sm font-semibold text-zinc-700 hover:bg-zinc-50"
          >
            View products
          </Link>
        </div>
      </div>
    </div>
  );
}
