"use client";

import { AnimatedSection } from "./AnimatedSection";

export function ProblemStats() {
  const stats = [
    {
      value: "$4.5T",
      label: "Lost annually to counterfeit goods",
      icon: (
        <svg
          className="h-8 w-8 text-[#FF6B35]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
          />
        </svg>
      ),
    },
    {
      value: "73%",
      label: "Of consumers don't trust product claims",
      icon: (
        <svg
          className="h-8 w-8 text-[#FF6B35]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
          />
        </svg>
      ),
    },
    {
      value: "$40B+",
      label: "Lost annually in supply chain fraud",
      icon: (
        <svg
          className="h-8 w-8 text-[#FF6B35]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      ),
    },
  ];

  return (
    <section className="bg-gray-50 py-24 sm:py-28">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <AnimatedSection>
          <div className="mx-auto max-w-2xl text-center">
            <h2 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
              The problem is real
            </h2>
            <p className="mt-4 text-lg leading-7 text-gray-600">
              Modern supply chains face critical trust challenges that cost
              trillions and put lives at risk.
            </p>
          </div>
        </AnimatedSection>
        <div className="mx-auto mt-16 grid max-w-7xl grid-cols-1 gap-8 sm:grid-cols-3">
          {stats.map((stat, index) => (
            <AnimatedSection key={index} delay={index * 100}>
              <div className="flex flex-col items-center rounded-lg bg-white p-6 text-center shadow-sm hover:shadow-md transition-all duration-300 hover:-translate-y-1 hover:scale-105">
              <div className="mb-4">{stat.icon}</div>
              <div className="text-4xl font-bold text-[#1A1A1A]">
                {stat.value}
              </div>
                <div className="mt-2 text-sm text-gray-600">{stat.label}</div>
              </div>
            </AnimatedSection>
          ))}
        </div>
      </div>
    </section>
  );
}
