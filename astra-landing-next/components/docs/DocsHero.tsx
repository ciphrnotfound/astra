'use client';

import { motion } from 'framer-motion';

export default function DocsHero() {
  return (
    <section className="relative pt-40 pb-20 px-6">
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:48px_48px] pointer-events-none" />
      
      <div className="max-w-4xl mx-auto relative text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
        >
          <div className="inline-flex items-center gap-2 px-3 py-1 border border-gray-200 bg-white text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-8">
            Documentation
          </div>
          
          <h1 className="text-5xl md:text-7xl font-medium tracking-tight text-gray-900 leading-tight mb-8" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Get started
            <br />
            <span className="text-gray-600">with Astra.</span>
          </h1>

          <p className="text-xl text-gray-600 max-w-3xl mx-auto leading-relaxed">
            Install Astra CLI and start migrating code across languages in minutes.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
