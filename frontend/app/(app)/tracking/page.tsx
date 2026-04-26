import dynamic from "next/dynamic";
import { LoadingSpinner } from "@/components/ui/loading-spinner";

// Code-split: EventTrackingForm includes validation + contract dependencies
const EventTrackingForm = dynamic(
  () => import("@/components/forms/EventTrackingForm"),
  {
    loading: () => (
      <div className="flex items-center justify-center py-24">
        <LoadingSpinner />
      </div>
    ),
  }
);

export default function TrackingPage() {
  return (
    <main className="mx-auto max-w-3xl px-6 py-10">
      <div className="text-center mb-10">
        <h1 className="text-3xl font-bold text-zinc-900">Event Tracking</h1>
        <p className="text-zinc-600 mt-2">Record a supply chain event on the Stellar blockchain.</p>
      </div>
      <EventTrackingForm />
    </main>
  );
}
