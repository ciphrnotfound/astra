'use client';

import { motion } from 'framer-motion';

export default function Mission() {
  return (
    <section className="py-32 px-6 bg-white">
      <div className="max-w-6xl mx-auto">
        <div className="grid md:grid-cols-2 gap-16 items-center">
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
          >
            <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              The mission
            </h2>
            <p className="text-gray-600 text-lg leading-relaxed mb-6">
              Developers shouldn't have to rewrite entire codebases when migrating between languages. Astra uses AI-powered semantic understanding to automate the tedious parts while preserving your code's logic and structure.
            </p>
            <p className="text-gray-600 text-lg leading-relaxed">
              Built in the open, Astra is free forever and welcomes contributions from the community. Whether you're migrating TypeScript to Rust or Python to Go, Astra makes it fast and reliable.
            </p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: 20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="relative"
          >
            <div className="aspect-square rounded-2xl bg-gray-100 border border-gray-200" />
          </motion.div>
        </div>
      </div>
    </section>
  );
}
