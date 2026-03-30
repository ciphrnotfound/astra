'use client';

import { motion } from 'framer-motion';
import { ArrowRight } from 'lucide-react';

const CTA = () => {
  return (
    <section className="py-32 bg-[#0f172a]">
      <div className="max-w-4xl mx-auto px-6 text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="space-y-8"
        >
          <h2 className="text-4xl md:text-5xl font-semibold text-white">
            Ready to transform your workflow?
          </h2>
          <p className="text-lg text-gray-400 max-w-2xl mx-auto">
            Join developers building better code with Astra
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <button className="bg-[#2B6BE7] text-white px-8 py-3 rounded-lg hover:bg-[#1a4db8] transition-all flex items-center space-x-2 font-medium">
              <span>Get started free</span>
              <ArrowRight size={18} />
            </button>
            <button className="border border-gray-700 text-white px-8 py-3 rounded-lg hover:bg-gray-800 transition-all font-medium">
              View documentation
            </button>
          </div>
        </motion.div>
      </div>
    </section>
  );
};

export default CTA;
