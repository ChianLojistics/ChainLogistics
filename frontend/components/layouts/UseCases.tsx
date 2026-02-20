"use client";

import { AnimatedSection } from "./AnimatedSection";

export function UseCases() {
  const useCases = [
    {
      title: "Food & Agriculture",
      description:
        "Verify organic certifications, track farm-to-table journeys, and ensure food safety. Consumers can see exactly where their food comes from.",
      examples: ["Organic produce", "Fair-trade coffee", "Sustainable seafood"],
      icon: (
        <svg
          className="h-6 w-6 text-[#00D4AA]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
          />
        </svg>
      ),
    },
    {
      title: "Pharmaceuticals",
      description:
        "Combat counterfeit medications that kill 250,000+ people annually. Verify drug authenticity and ensure patient safety with immutable records.",
      examples: ["Prescription drugs", "Vaccines", "Medical devices"],
      icon: (
        <svg
          className="h-6 w-6 text-[#00D4AA]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z"
          />
        </svg>
      ),
    },
    {
      title: "Electronics",
      description:
        "Ensure ethical sourcing of conflict minerals, verify manufacturing locations, and prevent counterfeit electronics from entering the market.",
      examples: ["Smartphones", "Laptops", "Components"],
      icon: (
        <svg
          className="h-6 w-6 text-[#00D4AA]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"
          />
        </svg>
      ),
    },
    {
      title: "Luxury Goods",
      description:
        "Protect brands from counterfeits and give customers confidence in their purchases. Verify authenticity of high-value items with blockchain proof.",
      examples: ["Watches", "Handbags", "Jewelry"],
      icon: (
        <svg
          className="h-6 w-6 text-[#00D4AA]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"
          />
        </svg>
      ),
    },
  ];

  return (
    <section id="use-cases" className="bg-white py-24 sm:py-28">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <AnimatedSection>
          <div className="mx-auto max-w-2xl text-center">
            <h2 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
              Who uses ChainLojistic
            </h2>
            <p className="mt-4 text-lg leading-7 text-gray-600">
              Trusted by producers, manufacturers, and consumers across industries.
            </p>
          </div>
        </AnimatedSection>
        <div className="mx-auto mt-16 grid max-w-7xl grid-cols-1 gap-8 lg:grid-cols-2">
          {useCases.map((useCase, index) => (
            <AnimatedSection key={index} delay={index * 100}>
              <div className="flex flex-col rounded-lg border border-gray-200 bg-white p-8 shadow-sm hover:shadow-md transition-all duration-300 hover:-translate-y-2 hover:scale-[1.01]">
              <div className="mb-4 flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-[#00D4AA]/10">
                  {useCase.icon}
                </div>
                <h3 className="text-xl font-semibold text-[#1A1A1A]">
                  {useCase.title}
                </h3>
              </div>
              <p className="mb-4 text-sm text-gray-600">
                {useCase.description}
              </p>
              <div className="mt-auto flex flex-wrap gap-2">
                {useCase.examples.map((example, i) => (
                  <span
                    key={i}
                    className="rounded-full bg-[#F5F5F5] px-3 py-1 text-xs font-medium text-gray-700"
                  >
                    {example}
                  </span>
                ))}
                </div>
              </div>
            </AnimatedSection>
          ))}
        </div>
      </div>
    </section>
  );
}
