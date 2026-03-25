"use client";

import { useEffect } from "react";
import { trackError } from "@/lib/analytics";

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error(error);
    trackError(error, {
      source: "GlobalError",
      digest: error.digest,
    });
  }, [error]);

  return (
    <html lang="en">
      <body className="min-h-screen bg-zinc-50">
        <div className="mx-auto flex min-h-screen max-w-2xl flex-col items-center justify-center px-6 py-16 text-center">
          <h1 className="text-2xl font-bold text-zinc-900">Something went wrong</h1>
          <p className="mt-2 text-sm text-zinc-600">
            The app hit an unexpected error. Try again, or refresh the page if it persists.
          </p>
          {error?.message ? (
            <p className="mt-4 text-xs text-zinc-500 break-words">{error.message}</p>
          ) : null}
          <div className="mt-6 flex items-center justify-center gap-3">
            <button
              type="button"
              onClick={() => reset()}
              className="rounded-lg bg-zinc-900 px-4 py-2 text-sm font-semibold text-white hover:bg-zinc-800"
            >
              Try again
            </button>
            <button
              type="button"
              onClick={() => window.location.reload()}
              className="rounded-lg border border-zinc-300 bg-white px-4 py-2 text-sm font-semibold text-zinc-700 hover:bg-zinc-50"
            >
              Refresh
            </button>
          </div>
        </div>
      </body>
    </html>
  );
}
