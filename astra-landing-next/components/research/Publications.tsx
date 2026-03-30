'use client';

import { motion } from 'framer-motion';

const publications = [
  {
    title: 'Type-Safe Cross-Language Code Migration Using Semantic Analysis',
    authors: 'Chen, A., Kim, S., Johnson, M.',
    venue: 'ICSE 2026',
    year: '2026',
  },
  {
    title: 'Formal Verification of AI-Generated Code Transformations',
    authors: 'Rodriguez, E., Chen, A.',
    venue: 'PLDI 2025',
    year: '2025',
  },
  {
    title: 'Context-Aware Program Synthesis for Multi-Language Codebases',
    authors: 'Kim, S., Johnson, M., Chen, A.',
    venue: 'OOPSLA 2025',
    year: '2025',
  },
];

export default function Publications() {
  return (
    <section className="py-32 px-6 bg-white">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-20">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            whileInView={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5 }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-gray-200 bg-gray-50 text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-8"
          >
            Publications
          </motion.div>
          
          <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Recent publications
          </h2>
        </div>

        <div className="space-y-6">
          {publications.map((pub, index) => (
            <motion.div
              key={pub.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: index * 0.1 }}
              className="p-8 rounded-2xl bg-gray-50 border border-gray-200 hover:bg-white hover:shadow-lg transition-all"
            >
              <h3 className="text-lg font-medium text-gray-900 mb-2">
                {pub.title}
              </h3>
              <p className="text-sm text-gray-600 mb-2">
                {pub.authors}
              </p>
              <div className="flex items-center gap-3 text-xs text-gray-500">
                <span>{pub.venue}</span>
                <span>•</span>
                <span>{pub.year}</span>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
