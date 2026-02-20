"use client";

import Link from "next/link";
import { useState, useEffect } from "react";

export function Navigation() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 20);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  const handleNavClick = (e: React.MouseEvent<HTMLAnchorElement>, href: string) => {
    if (href.startsWith("#")) {
      e.preventDefault();
      const element = document.querySelector(href);
      if (element) {
        const offset = 80; // navbar height + padding
        const elementPosition = element.getBoundingClientRect().top;
        const offsetPosition = elementPosition + window.pageYOffset - offset;

        window.scrollTo({
          top: offsetPosition,
          behavior: "smooth",
        });
      }
    }
  };

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled
          ? "bg-white/95 backdrop-blur-md shadow-sm border-b border-gray-200"
          : "bg-white/80 backdrop-blur-sm border-b border-gray-100"
      }`}
    >
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          <Link
            href="/"
            className="text-xl font-bold text-gray-900 hover:text-[#0066FF] transition-colors"
          >
            ChainLojistic
          </Link>
          <div className="hidden md:flex md:items-center md:gap-8">
            <Link
              href="#features"
              onClick={(e) => handleNavClick(e, "#features")}
              className="text-sm font-medium text-gray-700 hover:text-[#0066FF] transition-colors duration-200"
            >
              Features
            </Link>
            <Link
              href="#how-it-works"
              onClick={(e) => handleNavClick(e, "#how-it-works")}
              className="text-sm font-medium text-gray-700 hover:text-[#0066FF] transition-colors duration-200"
            >
              How It Works
            </Link>
            <Link
              href="#use-cases"
              onClick={(e) => handleNavClick(e, "#use-cases")}
              className="text-sm font-medium text-gray-700 hover:text-[#0066FF] transition-colors duration-200"
            >
              Use Cases
            </Link>
            <Link
              href="/register"
              className="rounded-lg bg-[#0066FF] px-5 py-2 text-sm font-semibold text-white shadow-md shadow-blue-500/25 hover:bg-[#0052CC] hover:shadow-lg transition-all duration-200"
            >
              Get Started
            </Link>
          </div>
          <button
            className="md:hidden text-[#1A1A1A]"
            aria-label="Open menu"
            aria-expanded="false"
          >
            <svg
              className="h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 6h16M4 12h16M4 18h16"
              />
            </svg>
          </button>
        </div>
      </div>
    </nav>
  );
}
