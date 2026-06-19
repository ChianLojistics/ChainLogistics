"use client";

import Link from "next/link";
import { useState, useEffect, useCallback } from "react";

export function Navigation() {
  const [scrolled, setScrolled] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 20);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  // Lock body scroll when mobile menu is open
  useEffect(() => {
    if (mobileMenuOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [mobileMenuOpen]);

  // Close mobile menu on Escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && mobileMenuOpen) {
        setMobileMenuOpen(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [mobileMenuOpen]);

  const closeMobileMenu = useCallback(() => {
    setMobileMenuOpen(false);
  }, []);

  const handleNavClick = (e: React.MouseEvent<HTMLAnchorElement>, href: string) => {
    closeMobileMenu();
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
      aria-label="Main navigation"
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
            onClick={closeMobileMenu}
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
            className="md:hidden text-[#1A1A1A] hover:text-[#0066FF] transition-colors"
            aria-label={mobileMenuOpen ? "Close menu" : "Open menu"}
            aria-expanded={mobileMenuOpen}
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
          >
            {mobileMenuOpen ? (
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
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            ) : (
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
            )}
          </button>
        </div>
      </div>

      {/* Mobile menu overlay */}
      {mobileMenuOpen && (
        <div className="md:hidden">
          {/* Backdrop */}
          <div
            className="fixed inset-0 top-16 bg-black/20 backdrop-blur-sm"
            aria-hidden="true"
            onClick={closeMobileMenu}
          />
          {/* Menu panel */}
          <div className="absolute top-full left-0 right-0 border-b border-gray-200 bg-white shadow-lg">
            <div className="space-y-1 px-6 py-4">
              <Link
                href="#features"
                onClick={(e) => handleNavClick(e, "#features")}
                className="block rounded-md px-3 py-3 text-base font-medium text-gray-700 hover:bg-blue-50 hover:text-[#0066FF] transition-colors"
              >
                Features
              </Link>
              <Link
                href="#how-it-works"
                onClick={(e) => handleNavClick(e, "#how-it-works")}
                className="block rounded-md px-3 py-3 text-base font-medium text-gray-700 hover:bg-blue-50 hover:text-[#0066FF] transition-colors"
              >
                How It Works
              </Link>
              <Link
                href="#use-cases"
                onClick={(e) => handleNavClick(e, "#use-cases")}
                className="block rounded-md px-3 py-3 text-base font-medium text-gray-700 hover:bg-blue-50 hover:text-[#0066FF] transition-colors"
              >
                Use Cases
              </Link>
              <div className="pt-2">
                <Link
                  href="/register"
                  onClick={closeMobileMenu}
                  className="block w-full rounded-lg bg-[#0066FF] px-3 py-3 text-center text-base font-semibold text-white shadow-md hover:bg-[#0052CC] transition-all duration-200"
                >
                  Get Started
                </Link>
              </div>
            </div>
          </div>
        </div>
      )}
    </nav>
  );
}
