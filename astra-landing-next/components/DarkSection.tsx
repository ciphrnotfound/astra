'use client';

import { motion } from 'framer-motion';
import { ArrowRight } from 'lucide-react';

const DarkSection = () => {
  return (
    <section className="py-32 px-6 bg-[#faf9f6]">
      <div className="max-w-4xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="relative p-16 rounded-2xl bg-white border border-gray-200 text-center overflow-hidden"
        >
          {/* Subtle grid background */}
          <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808008_1px,transparent_1px),linear-gradient(to_bottom,#80808008_1px,transparent_1px)] bg-[size:32px_32px] pointer-events-none" />
          
          <div className="relative">
            <motion.div
              initial={{ opacity: 0, scale: 0.95 }}
              whileInView={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.5, delay: 0.2 }}
              className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-gray-200 bg-gray-50 text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-8"
            >
              Get Started
            </motion.div>
            
            <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Ready to transform
              <br />
              <span className="text-gray-600">your codebase?</span>
            </h2>
            
            <p className="text-gray-600 text-lg mb-10 max-w-2xl mx-auto">
              Join thousands of developers using Astra to migrate, refactor, and debug code across languages.
            </p>

            <div className="flex flex-col sm:flex-row items-center justify-center gap-3">
              <button className="relative group overflow-hidden bg-gray-900 text-white px-6 py-3 rounded text-sm font-medium transition-all w-full sm:w-auto hover:shadow-lg">
                <span className="relative z-10 inline-flex items-center gap-2">
                  Start building for free
                  <ArrowRight className="w-4 h-4 group-hover:translate-x-0.5 transition-transform" />
                </span>
                <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
              </button>
              <button className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 rounded text-sm font-medium transition-all w-full sm:w-auto hover:text-white">
                <span className="relative z-10">Talk to sales</span>
                <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
              </button>
            </div>

            <p className="text-xs text-gray-500 mt-6">
              No credit card required • Free forever for individuals
            </p>
          </div>
        </motion.div>
      </div>
    </section>
  );
};

export default DarkSection;
