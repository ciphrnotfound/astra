'use client';

import { motion } from 'framer-motion';
import { ChevronDown, Github } from 'lucide-react';

const Navbar = () => {
  return (
    <motion.nav
      className="fixed top-0 left-0 right-0 z-50 bg-[#faf9f6]/80 backdrop-blur-sm border-b border-gray-200"
      initial={{ y: -100 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
    >
      <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
        <div className="flex items-center gap-8">
          <div className="flex items-center gap-3">
            {/* Astra Logo - Square brand icon */}
            <div className="md:w-30 md:h-30 h-16 w-16 sm:h-10 sm:w-10 flex items-center justify-center shrink-0">
              <img src="/astra-logo-2.png" alt="Astra Logo" className="w-full h-full object-contain" />
            </div>
            
          </div>
          
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

        <div className="flex items-center gap-4">
          <a
            href="https://github.com/yourusername/astra"
            target="_blank"
            rel="noopener noreferrer"
            className="relative group overflow-hidden flex items-center gap-2 px-3 py-1.5 border border-gray-300 text-gray-600 transition-all text-sm hover:text-white"
          >
            <span className="relative z-10 flex items-center gap-2">
              <Github className="w-4 h-4" />
              <span className="font-medium">Star</span>
              <span className="px-1.5 py-0.5 bg-gray-100 text-gray-700 group-hover:bg-gray-800 group-hover:text-white transition-colors text-xs font-bold">0</span>
            </span>
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </a>
          <a
            href="https://x.com/yourusername"
            target="_blank"
            rel="noopener noreferrer"
            className="relative group overflow-hidden w-9 h-9 border border-gray-300 flex items-center justify-center text-gray-600 transition-all hover:text-white"
          >
            <svg className="w-4 h-4 relative z-10" viewBox="0 0 24 24" fill="currentColor">
              <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
            </svg>
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </a>
          <button className="relative group overflow-hidden border border-gray-900 text-gray-900 text-sm font-medium px-4 py-2 transition-all hover:text-white">
            <span className="relative z-10">Get started</span>
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </button>
        </div>
      </div>
    </motion.nav>
  );
};

export default Navbar;
