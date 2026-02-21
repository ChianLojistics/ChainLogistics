import { NavBar } from "@/components/layouts";

export default function AppLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-zinc-50 flex flex-col">
      <NavBar />
      <main className="flex-1">{children}</main>
    </div>
  );
}
