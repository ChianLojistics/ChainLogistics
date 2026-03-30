"use client";

import Link from "next/link";
import { AnimatedSection } from "./AnimatedSection";

export function CTA() {
  return (
    <section className="bg-white py-24 sm:py-28">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <AnimatedSection>
          <div className="mx-auto max-w-4xl overflow-hidden rounded-2xl bg-gradient-to-r from-[#0066FF] via-[#0052CC] to-[#00D4AA] px-8 py-20 text-center shadow-2xl shadow-blue-500/20 hover:shadow-3xl transition-shadow duration-500">
          <h2 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">
            Ready to bring transparency to your supply chain?
          </h2>
          <p className="mx-auto mt-6 max-w-xl text-lg leading-8 text-white/90">
            Join producers, manufacturers, and consumers who trust ChainLojistic
            for complete supply chain visibility.
          </p>
          <div className="mt-10 flex items-center justify-center gap-4">
            <Link
              href="/register"
              className="rounded-lg bg-white px-6 py-3 text-base font-semibold text-[#0066FF] shadow-lg hover:bg-gray-50 transition-all duration-200 hover:scale-105"
            >
              Get Started Free
            </Link>
            <Link
              href="#how-it-works"
              className="rounded-lg border-2 border-white px-6 py-3 text-base font-semibold text-white hover:bg-white/10 transition-all duration-200"
            >
              View Demo
            </Link>
          </div>
          </div>
        </AnimatedSection>
      </div>
    </section>
  );
}
