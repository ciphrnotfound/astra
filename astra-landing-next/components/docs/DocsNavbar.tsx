'use client';

import Link from 'next/link';

export default function DocsNavbar() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-white border-b border-gray-200">
      <div className="flex items-center justify-between px-6 py-4">
        {/* Logo */}
        <Link href="/" className="flex items-center gap-2">
          <div className="w-12 h-12 flex items-center justify-center shrink-0">
            <img src="/astra-logo-2.png" alt="Astra Logo" className="w-full h-full object-contain" />
          </div>
          
        </Link>

        {/* Center - Search and Ask AI */}
        <div className="flex items-center gap-3 flex-1 max-w-2xl mx-8">
          {/* Search Input */}
          <div className="relative flex-1">
            <input
              type="text"
              placeholder="Search documentation..."
              className="w-full px-4 py-2 pl-10 border border-gray-200 bg-[#faf9f6] text-sm focus:outline-none focus:ring-2 focus:ring-gray-900 focus:border-transparent transition-all"
            />
            <svg
              className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
          </div>

          {/* Ask AI Button */}
          <button className="relative group overflow-hidden bg-gray-900 text-white px-4 py-2 text-sm font-medium transition-all hover:shadow-lg whitespace-nowrap">
            <span className="relative z-10 flex items-center gap-2">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              Ask AI
            </span>
            <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
          </button>
        </div>

        {/* Right Side - Support and Dashboard */}
        <div className="flex items-center gap-4">
          {/* Support Link */}
          <Link href="/support" className="text-sm text-gray-600 hover:text-gray-900 transition-colors">
            Support
          </Link>

          {/* Dashboard Button */}
          <button className="relative group overflow-hidden border border-gray-900 text-gray-900 px-4 py-2 text-sm font-medium transition-all hover:text-white">
            <span className="relative z-10">Dashboard</span>
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </button>
        </div>
      </div>
    </nav>
  );
}
