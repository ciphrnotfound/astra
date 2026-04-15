'use client';

import { motion } from 'framer-motion';

export default function Metrics() {
  const metrics = [
    { value: '100k+', label: 'Developers', sublabel: 'Using Astra daily' },
    { value: '1M+', label: 'Codebases', sublabel: 'Indexed and analyzed' },
    { value: '<1s', label: 'Search', sublabel: 'Sub-second retrieval' },
    { value: '100%', label: 'Local', sublabel: 'Privacy by design' },
  ];

  return (
    <section className="py-16 md:py-32 px-4 md:px-6 bg-white">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-12 md:mb-20"
        >
          <h2 className="text-2xl md:text-4xl lg:text-5xl font-medium text-gray-900 mb-4 md:mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Trusted by developers
            <br />
            <span className="text-gray-600">around the world</span>
          </h2>
        </motion.div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-6 md:gap-8">
          {metrics.map((metric, index) => (
            <motion.div
              key={metric.label}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              className="text-center p-6 border border-gray-200 bg-[#faf9f6] hover:border-gray-900 transition-all duration-300 hover:-translate-y-1 group"
            >
              <div className="text-3xl md:text-5xl font-medium text-gray-900 mb-2 transition-transform duration-300 group-hover:scale-110" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                {metric.value}
              </div>
              <div className="text-sm md:text-base text-gray-900 mb-1">{metric.label}</div>
              <div className="text-xs text-gray-600">{metric.sublabel}</div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
