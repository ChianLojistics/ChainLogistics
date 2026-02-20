import {
  Navigation,
  Hero,
  ProblemStats,
  Features,
  HowItWorks,
  UseCases,
  TrustBlockchain,
  CTA,
  Footer,
} from "@/components/layouts";

export default function Home() {
  return (
    <div className="min-h-screen bg-white">
      <Navigation />
      <main className="bg-white">
        <Hero />
        <ProblemStats />
        <Features />
        <HowItWorks />
        <UseCases />
        <TrustBlockchain />
        <CTA />
      </main>
      <Footer />
    </div>
  );
}
