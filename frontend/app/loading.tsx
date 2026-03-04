import { LoadingSpinner } from "@/components/ui/loading-spinner";

export default function Loading() {
  return (
    <div className="min-h-screen bg-zinc-50">
      <div className="mx-auto flex min-h-screen max-w-2xl flex-col items-center justify-center px-6 py-16 text-center">
        <LoadingSpinner size={28} />
        <p className="mt-4 text-sm text-zinc-600">Loading…</p>
      </div>
    </div>
  );
}
