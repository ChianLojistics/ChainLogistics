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
      <main id="main-content">
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
