'use client';

import { motion } from 'framer-motion';
import { ArrowRight, Sparkles } from 'lucide-react';
import HeroCli from './HeroCli';

export default function Hero() {
  return (
    <section className="relative pt-24 sm:pt-32 md:pt-40 pb-12 sm:pb-16 md:pb-20 px-4 sm:px-6 overflow-hidden">
      {/* Grid lines background */}
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:48px_48px] pointer-events-none" />
      
      <div className="max-w-6xl mx-auto relative">
        <div className="text-center mb-12 sm:mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="space-y-4 sm:space-y-6"
          >
            {/* Breadcrumb/Category */}
            <div className="flex items-center justify-center gap-2 text-xs sm:text-sm text-gray-500">
              <Sparkles className="w-3 h-3 sm:w-4 sm:h-4" />
              <span className="font-medium">AI Development Tools</span>
            </div>

            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-100 border border-gray-200 text-gray-700 text-xs font-medium">
              <span className="w-1.5 h-1.5 rounded-full bg-gray-900 animate-pulse"></span>
              Introducing Astra v2.0
            </div>

            <h1 className="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-medium tracking-tight text-gray-900 leading-tight px-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Time travel debugging
              <br />
              <span className="text-gray-600">meets AI migration</span>
            </h1>

            <p className="text-base sm:text-lg text-gray-600 max-w-2xl mx-auto leading-relaxed px-4">
              AI-powered CLI that migrates code across languages, hunts security vulnerabilities, 
              and lets you rewind execution history. Built for teams who ship fast.
            </p>

            <div className="flex flex-row items-center justify-center gap-2 sm:gap-3 pt-4 px-4">
              <a href="/signup" className="relative group overflow-hidden bg-gray-900 text-white px-3 sm:px-6 py-1.5 sm:py-2.5 rounded text-xs sm:text-sm font-medium transition-all hover:shadow-lg text-center">
                <span className="relative z-10 inline-flex items-center justify-center gap-1 sm:gap-2">
                  Get started
                  <ArrowRight className="w-3 h-3 sm:w-4 sm:h-4 group-hover:translate-x-0.5 transition-transform" />
                </span>
                <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
              </a>
              <a href="/docs" className="relative group overflow-hidden border border-gray-900 text-gray-900 px-3 sm:px-6 py-1.5 sm:py-2.5 rounded text-xs sm:text-sm font-medium transition-all hover:text-white text-center">
                <span className="relative z-10">Documentation</span>
                <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
              </a>
            </div>

            {/* Terminal command preview */}
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.3 }}
              className="mt-6 sm:mt-8 inline-flex items-center gap-2 px-3 sm:px-4 py-2 bg-gray-50 border border-gray-200 rounded text-xs sm:text-sm font-mono text-gray-700 max-w-full overflow-x-auto"
            >
              <span className="text-gray-400">$</span>
              <span className="whitespace-nowrap">astra :bisect "login crashes on empty email"</span>
            </motion.div>
          </motion.div>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2 }}
          className="relative"
        >
          <HeroCli />
          
          {/* Decorative corner accents - hidden on mobile */}
          <div className="hidden sm:block absolute -top-4 -left-4 w-8 h-8 border-l-2 border-t-2 border-gray-200 pointer-events-none" />
          <div className="hidden sm:block absolute -top-4 -right-4 w-8 h-8 border-r-2 border-t-2 border-gray-200 pointer-events-none" />
          <div className="hidden sm:block absolute -bottom-4 -left-4 w-8 h-8 border-l-2 border-b-2 border-gray-200 pointer-events-none" />
          <div className="hidden sm:block absolute -bottom-4 -right-4 w-8 h-8 border-r-2 border-b-2 border-gray-200 pointer-events-none" />
        </motion.div>
      </div>
    </section>
  );
}
