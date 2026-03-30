'use client';

import { motion } from 'framer-motion';
import { Github, Twitter } from 'lucide-react';

export default function Team() {
  return (
    <section className="py-32 px-6 bg-white">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            whileInView={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5 }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-gray-200 bg-gray-50 text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-8"
          >
            Creator
          </motion.div>
          
          <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Built by Shay Jeremy
          </h2>
          <p className="text-gray-600 text-lg max-w-2xl mx-auto">
            Solo developer passionate about making code migration accessible and open source.
          </p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="max-w-md mx-auto"
        >
          <div className="p-8 rounded-2xl bg-gray-50 border border-gray-200 text-center">
            <div className="w-32 h-32 rounded-full bg-gray-200 border border-gray-300 mx-auto mb-6" />
            <h3 className="text-2xl font-medium text-gray-900 mb-2" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Shay Jeremy
            </h3>
            <p className="text-gray-600 mb-6">
              Creator & Maintainer
            </p>
            <div className="flex items-center justify-center gap-3">
              <a
                href="https://github.com/yourusername"
                target="_blank"
                rel="noopener noreferrer"
                className="w-10 h-10 border border-gray-300 flex items-center justify-center text-gray-600 hover:border-gray-900 hover:text-gray-900 transition-colors"
              >
                <Github className="w-4 h-4" />
              </a>
              <a
                href="https://x.com/yourusername"
                target="_blank"
                rel="noopener noreferrer"
                className="w-10 h-10 border border-gray-300 flex items-center justify-center text-gray-600 hover:border-gray-900 hover:text-gray-900 transition-colors"
              >
                <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
                </svg>
              </a>
            </div>
          </div>

          <div className="mt-12 p-6 rounded-2xl bg-white border border-gray-200 text-center">
            <h3 className="text-lg font-medium text-gray-900 mb-2">
              Want to contribute?
            </h3>
            <p className="text-gray-600 mb-4">
              Astra is open source and welcomes contributions from the community.
            </p>
            <a
              href="https://github.com/yourusername/astra"
              target="_blank"
              rel="noopener noreferrer"
              className="relative group overflow-hidden border border-gray-900 text-gray-900 text-sm font-medium px-6 py-2.5 inline-flex items-center gap-2 transition-all hover:text-white"
            >
              <span className="relative z-10">View on GitHub</span>
              <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
            </a>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
