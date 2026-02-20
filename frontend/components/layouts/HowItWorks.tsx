"use client";

import { AnimatedSection } from "./AnimatedSection";

export function HowItWorks() {
  const steps = [
    {
      number: "1",
      title: "Register Product",
      description:
        "Producers register products at origin with complete details. Each product gets a unique blockchain ID and cryptographic proof of authenticity.",
      icon: (
        <svg
          className="h-8 w-8 text-[#00D4AA]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
      ),
    },
    {
      number: "2",
      title: "Track Events",
      description:
        "As products move through the supply chain, each party adds events: processing, shipping, quality checks. All with timestamps and location data.",
      icon: (
        <svg
          className="h-8 w-8 text-[#00D4AA]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M13 10V3L4 14h7v7l9-11h-7z"
          />
        </svg>
      ),
    },
    {
      number: "3",
      title: "Verify & Trust",
      description:
        "Consumers scan QR codes to see the complete product journey. Verify authenticity, check certifications, and make informed decisions.",
      icon: (
        <svg
          className="h-8 w-8 text-[#00D4AA]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      ),
    },
  ];

  return (
    <section id="how-it-works" className="bg-gray-50 py-24 sm:py-28">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <AnimatedSection>
          <div className="mx-auto max-w-2xl text-center">
            <h2 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
              How it works
            </h2>
            <p className="mt-4 text-lg leading-7 text-gray-600">
              Simple, transparent, and secure. Three steps to complete supply
              chain visibility.
            </p>
          </div>
        </AnimatedSection>
        <div className="mx-auto mt-16 max-w-5xl">
          <div className="grid grid-cols-1 gap-8 lg:grid-cols-3">
            {steps.map((step, index) => (
              <AnimatedSection key={index} delay={index * 150}>
                <div className="relative">
                {index < steps.length - 1 && (
                  <div className="absolute top-12 left-full hidden h-0.5 w-full bg-gray-300 lg:block">
                    <div className="absolute right-0 top-1/2 h-2 w-2 -translate-y-1/2 translate-x-1/2 rounded-full bg-[#00D4AA]"></div>
                  </div>
                )}
                <div className="flex flex-col items-center text-center">
                  <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-[#00D4AA]/10">
                    {step.icon}
                  </div>
                  <div className="mb-2 text-2xl font-bold text-[#0066FF]">
                    {step.number}
                  </div>
                  <h3 className="text-xl font-semibold text-[#1A1A1A]">
                    {step.title}
                  </h3>
                    <p className="mt-2 text-sm text-gray-600">
                      {step.description}
                    </p>
                  </div>
                </div>
              </AnimatedSection>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
