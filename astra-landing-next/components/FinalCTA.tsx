'use client';

import { motion } from 'framer-motion';

const FinalCTA = () => {
  return (
    <section className="py-32 bg-white px-6">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8 }}
          className="bg-gray-900 p-16 md:p-20 text-center border border-gray-900"
        >
           <h2 className="text-4xl md:text-5xl font-medium text-white mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Ready to transform
              <br />
              <span className="text-gray-400">your codebase?</span>
           </h2>
           <p className="text-lg text-gray-400 mb-12 max-w-2xl mx-auto">
              Join thousands of developers using Astra for seamless code migrations.
           </p>

           <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
              <button className="relative group overflow-hidden bg-white text-gray-900 px-8 py-4 text-sm font-medium transition-all hover:shadow-xl">
                 <span className="relative z-10">Get started for free</span>
                 <div className="absolute inset-0 bg-gray-100 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
              </button>
              <button className="text-white text-sm font-medium hover:text-gray-300 transition-colors">
                 Talk to sales →
              </button>
           </div>
        </motion.div>
      </div>
    </section>
  );
};

export default FinalCTA;
