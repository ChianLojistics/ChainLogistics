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

export default function MarketingHomePage() {
  return (
    <>
      <Navigation />
      <main>
        <Hero />
        <ProblemStats />
        <Features />
        <HowItWorks />
        <UseCases />
        <TrustBlockchain />
        <CTA />
      </main>
      <Footer />
    </>
  );
}
