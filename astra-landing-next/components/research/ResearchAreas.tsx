'use client';

import { motion } from 'framer-motion';

const areas = [
  {
    title: 'Semantic Code Analysis',
    description: 'Understanding code meaning beyond syntax through advanced static analysis and type inference.',
  },
  {
    title: 'Cross-Language Type Systems',
    description: 'Mapping type systems across languages to ensure type-safe migrations and transformations.',
  },
  {
    title: 'Program Synthesis',
    description: 'Using AI to generate correct, idiomatic code in target languages from source specifications.',
  },
  {
    title: 'Formal Verification',
    description: 'Proving correctness of code transformations through formal methods and property-based testing.',
  },
];

export default function ResearchAreas() {
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
            Research Areas
          </motion.div>
          
          <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            What we're working on
          </h2>
        </div>

        <div className="grid md:grid-cols-2 gap-8">
          {areas.map((area, index) => (
            <motion.div
              key={area.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: index * 0.1 }}
              className="p-8 rounded-2xl bg-white border border-gray-200 hover:shadow-lg transition-shadow"
            >
              <h3 className="text-xl font-medium text-gray-900 mb-3 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                {area.title}
              </h3>
              <p className="text-gray-600 leading-relaxed">
                {area.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
