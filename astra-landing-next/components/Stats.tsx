'use client';

import { motion } from 'framer-motion';

const stats = [
  {
    value: '10x',
    label: 'Faster migrations',
    description: 'Complete cross-language migrations in minutes, not weeks',
  },
  {
    value: '99.9%',
    label: 'Type accuracy',
    description: 'AI-powered semantic analysis ensures type-safe transformations',
  },
  {
    value: '50+',
    label: 'Languages supported',
    description: 'From TypeScript to Rust, Python to Go, and everything in between',
  },
];

export default function Stats() {
  return (
    <section className="py-24 px-6 bg-[#faf9f6]">
      <div className="max-w-6xl mx-auto">
        <div className="grid md:grid-cols-3 gap-12">
          {stats.map((stat, index) => (
            <motion.div
              key={stat.label}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              className="text-center"
            >
              <div className="text-5xl md:text-6xl font-semibold text-gray-900 mb-3" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
                {stat.value}
              </div>
              <div className="text-lg font-medium text-gray-900 mb-2">
                {stat.label}
              </div>
              <p className="text-sm text-gray-600 leading-relaxed">
                {stat.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
