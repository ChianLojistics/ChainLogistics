import { NavBar } from "@/components/layouts";
import { ErrorBoundary } from "@/components/ErrorBoundary";

export default function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-zinc-50 flex flex-col">
      <NavBar />
      <main id="main-content" className="flex-1">
        <ErrorBoundary>
          {children}
        </ErrorBoundary>
      </main>
    </div>
  );
}
