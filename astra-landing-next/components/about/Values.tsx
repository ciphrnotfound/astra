'use client';

import { motion } from 'framer-motion';

const values = [
  {
    title: 'Developer-first',
    description: 'We build tools that developers actually want to use. Clean APIs, great docs, and zero magic.',
  },
  {
    title: 'Open & transparent',
    description: 'We believe in open source and building in public. Our roadmap is public, our code is open.',
  },
  {
    title: 'Quality over speed',
    description: 'We ship when it\'s ready, not when it\'s rushed. Every release is production-grade.',
  },
  {
    title: 'Local-first',
    description: 'Your code stays yours. We prioritize privacy and security in everything we build.',
  },
];

export default function Values() {
  return (
    <section className="py-32 px-6 bg-[#faf9f6]">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-20">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            whileInView={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5 }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-gray-200 bg-white text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-8"
          >
            Our Values
          </motion.div>
          
          <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            What we stand for
          </h2>
        </div>

        <div className="grid md:grid-cols-2 gap-8">
          {values.map((value, index) => (
            <motion.div
              key={value.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: index * 0.1 }}
              className="p-8 rounded-2xl bg-white border border-gray-200"
            >
              <h3 className="text-xl font-medium text-gray-900 mb-3 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                {value.title}
              </h3>
              <p className="text-gray-600 leading-relaxed">
                {value.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
