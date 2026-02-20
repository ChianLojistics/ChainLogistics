"use client";

import Link from "next/link";
import { AnimatedSection } from "./AnimatedSection";

export function TrustBlockchain() {
  const trustPoints = [
    {
      title: "Open Source",
      description: "Transparent, auditable code. Built by the community, for the community.",
      icon: (
        <svg
          className="h-6 w-6 text-[#0066FF]"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
          />
        </svg>
      ),
    },
    {
      title: "Stellar Blockchain",
      description: "Powered by Stellar's Soroban smart contracts. Fast, low-cost, and reliable.",
      icon: (
        <svg
          className="h-6 w-6 text-[#0066FF]"
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
      title: "Immutable Records",
      description: "Once recorded, data cannot be altered. Complete transparency and auditability.",
      icon: (
        <svg
          className="h-6 w-6 text-[#0066FF]"
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
  ];

  return (
    <section className="bg-gray-50 py-24 sm:py-28">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <AnimatedSection>
          <div className="mx-auto max-w-2xl text-center">
            <h2 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
              Built on trust and transparency
            </h2>
            <p className="mt-4 text-lg leading-7 text-gray-600">
              ChainLojistic leverages blockchain technology to create an
              unbreakable chain of trust from producer to consumer.
            </p>
          </div>
        </AnimatedSection>
        <div className="mx-auto mt-16 grid max-w-7xl grid-cols-1 gap-8 lg:grid-cols-3">
          {trustPoints.map((point, index) => (
            <AnimatedSection key={index} delay={index * 100}>
              <div className="flex flex-col rounded-lg bg-white p-6 shadow-sm hover:shadow-md transition-all duration-300 hover:-translate-y-1">
              <div className="mb-4">{point.icon}</div>
              <h3 className="text-lg font-semibold text-[#1A1A1A]">
                {point.title}
              </h3>
                <p className="mt-2 text-sm text-gray-600">{point.description}</p>
              </div>
            </AnimatedSection>
          ))}
        </div>
        <AnimatedSection delay={300}>
          <div className="mt-12 text-center">
          <Link
            href="https://github.com/ChainLojistics/ChainLogistics"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 rounded-lg border-2 border-[#0066FF] px-6 py-3 text-base font-semibold text-[#0066FF] hover:bg-[#0066FF] hover:text-white transition-all"
          >
            <svg
              className="h-5 w-5"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                fillRule="evenodd"
                d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.17 6.839 9.49.5.092.682-.217.682-.482 0-.237-.008-.866-.013-1.7-2.782.603-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.463-1.11-1.463-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.578 9.578 0 0112 6.836c.85.004 1.705.114 2.504.336 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.167 22 16.418 22 12c0-5.523-4.477-10-10-10z"
                clipRule="evenodd"
              />
            </svg>
            View on GitHub
          </Link>
          </div>
        </AnimatedSection>
      </div>
    </section>
  );
}
