'use client';

import { motion } from 'framer-motion';

const Testimonials = () => {
  const reviews = [
    { name: 'Sarah Chen', handle: '@sarahc_dev', text: 'Astra has completely changed how I debug. The semantic search is like having a second brain.' },
    { name: 'Marc Fontenot', handle: '@mfontenot', text: 'Local-first memory is a game changer for security. We sync across the team without leaking our IP.' },
    { name: 'Elena Rossi', handle: '@erossi_eng', text: "The most intuitive AI infrastructure I've used. Clean, fast, and actually works." },
    { name: 'Alex Rivera', handle: '@arivera', text: "Standard RAG was always too slow for us. Astra's sub-second retrieval is exactly what we needed for our real-time agents." },
    { name: 'James Wilson', handle: '@jwilson_dev', text: 'The dual-card setup for dev vs teams is genius. Finally, pricing that makes sense.' },
    { name: 'Yuki Sato', handle: '@yuki_codes', text: 'Astra v2.0 is a massive leap forward. The context awareness is unmatched.' },
  ];

  return (
    <section className="py-32 px-6 bg-[#faf9f6]">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8 }}
          className="text-center mb-20"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-gray-200 bg-white text-xs font-medium text-gray-700 mb-6">
            Testimonials
          </div>
          
          <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Loved by builders
          </h2>
          <p className="text-gray-600 text-lg">
            Join 100k+ developers building the future with Astra
          </p>
        </motion.div>

        <div className="columns-1 md:columns-2 lg:columns-3 gap-6 space-y-6">
          {reviews.map((review, i) => (
            <motion.div
              key={i}
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: i * 0.1 }}
              className="break-inside-avoid"
            >
              <div className="p-6 border border-gray-200 bg-white transition-all duration-300 hover:border-gray-900">
                <div className="flex items-center gap-3 mb-4">
                  <div className="w-10 h-10 bg-gray-200" />
                  <div>
                    <div className="text-sm font-medium text-gray-900">{review.name}</div>
                    <div className="text-xs text-gray-500">{review.handle}</div>
                  </div>
                </div>
                <p className="text-gray-600 leading-relaxed text-sm">
                  {review.text}
                </p>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Testimonials;
