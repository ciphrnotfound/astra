'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown, Menu, X } from 'lucide-react';

const Navbar = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  return (
    <>
      <motion.nav
        className="fixed top-0 left-0 right-0 z-50 bg-[#faf9f6]/80 backdrop-blur-sm border-b border-gray-200"
        initial={{ y: -100 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4 }}
      >
        <div className="max-w-7xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-8">
            <div className="flex items-center gap-3">
              {/* Astra Logo */}
              <div className="h-8 w-8 sm:h-32 sm:w-32 flex items-center justify-center shrink-0">
                <img src="/astra-logo-2.png" alt="Astra Logo" className="w-full h-full object-contain" />
              </div>
            </div>
            
            {/* Desktop Navigation */}
            <div className="hidden md:flex items-center gap-6">
              <button className="flex items-center gap-1 text-sm text-gray-600 hover:text-gray-900 transition-colors relative group">
                Product
                <ChevronDown className="w-3 h-3" />
                <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
              </button>
              <a href="#features" className="text-sm text-gray-600 hover:text-gray-900 transition-colors relative group">
                Features
                <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
              </a>
              <a href="#pricing" className="text-sm text-gray-600 hover:text-gray-900 transition-colors relative group">
                Pricing
                <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
              </a>
              <a href="#docs" className="text-sm text-gray-600 hover:text-gray-900 transition-colors relative group">
                Docs
                <span className="absolute bottom-0 left-0 w-0 h-[1px] bg-gray-900 group-hover:w-full transition-all duration-300 ease-out"></span>
              </a>
            </div>
          </div>

          {/* Desktop Actions */}
          <div className="hidden md:flex items-center gap-4">
            <a
              href="https://github.com/ciphrnotfound/astra"
              target="_blank"
              rel="noopener noreferrer"
              className="relative group overflow-hidden flex items-center gap-2 px-3 py-1.5 border border-gray-300 text-gray-600 transition-all text-sm hover:text-white"
            >
              <span className="relative z-10 flex items-center gap-2">
                <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
                <span className="font-medium">Star</span>
                <span className="px-1.5 py-0.5 bg-gray-100 text-gray-700 group-hover:bg-gray-800 group-hover:text-white transition-colors text-xs font-bold">0</span>
              </span>
              <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
            </a>
            <a
              href="https://x.com/ciphrnotfound"
              target="_blank"
              rel="noopener noreferrer"
              className="relative group overflow-hidden w-9 h-9 border border-gray-300 flex items-center justify-center text-gray-600 transition-all hover:text-white"
            >
              <svg className="w-4 h-4 relative z-10" viewBox="0 0 24 24" fill="currentColor">
                <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
              </svg>
              <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
            </a>
            <a
              href="/signup"
              className="relative group overflow-hidden border border-gray-900 text-gray-900 text-sm font-medium px-4 py-2 transition-all hover:text-white"
            >
              <span className="relative z-10">Get started</span>
              <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
            </a>
          </div>

          {/* Mobile Menu Button */}
          <button
            onClick={() => setIsMenuOpen(!isMenuOpen)}
            className="md:hidden p-2 text-gray-900 hover:bg-gray-100 transition-colors"
            aria-label="Toggle menu"
          >
            {isMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
        </div>
      </motion.nav>

      {/* Mobile Menu */}
      <AnimatePresence>
        {isMenuOpen && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.3 }}
            className="fixed top-16 left-0 right-0 z-40 bg-[#faf9f6] border-b border-gray-200 md:hidden overflow-hidden"
          >
            <div className="px-4 py-6 space-y-4">
              {/* Mobile Navigation Links */}
              <div className="space-y-3">
                <button className="flex items-center justify-between w-full text-left text-gray-900 py-2 border-b border-gray-200">
                  <span className="text-sm font-medium">Product</span>
                  <ChevronDown className="w-4 h-4" />
                </button>
                <a href="#features" className="block text-sm text-gray-600 py-2 border-b border-gray-200" onClick={() => setIsMenuOpen(false)}>
                  Features
                </a>
                <a href="#pricing" className="block text-sm text-gray-600 py-2 border-b border-gray-200" onClick={() => setIsMenuOpen(false)}>
                  Pricing
                </a>
                <a href="#docs" className="block text-sm text-gray-600 py-2 border-b border-gray-200" onClick={() => setIsMenuOpen(false)}>
                  Docs
                </a>
              </div>

              {/* Mobile Actions */}
              <div className="pt-4 space-y-3">
                <a
                  href="https://github.com/ciphrnotfound/astra"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-center gap-2 w-full px-4 py-2.5 border border-gray-300 text-gray-900 text-sm font-medium"
                >
                  <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                  </svg>
                  Star on GitHub
                </a>
                <a
                  href="https://x.com/ciphrnotfound"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-center gap-2 w-full px-4 py-2.5 border border-gray-300 text-gray-900 text-sm font-medium"
                >
                  <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
                  </svg>
                  Follow on X
                </a>
                <a
                  href="/signup"
                  className="block w-full px-4 py-2.5 bg-gray-900 text-white text-sm font-medium text-center"
                  onClick={() => setIsMenuOpen(false)}
                >
                  Get started
                </a>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
};

export default Navbar;
