'use client';

import { motion } from 'framer-motion';
import { ArrowRight } from 'lucide-react';

const DarkSection = () => {
  return (
    <section className="py-16 md:py-32 px-4 md:px-6 bg-gray-900">
      <div className="max-w-4xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center"
        >
          <h2 className="text-2xl md:text-4xl lg:text-5xl font-medium text-white mb-4 md:mb-6 tracking-tight px-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Ready to give your codebase
            <br />
            <span className="text-gray-400">a permanent memory?</span>
          </h2>
          
          <p className="text-sm md:text-base lg:text-lg text-gray-400 mb-8 md:mb-10 max-w-2xl mx-auto px-4">
            Join thousands of developers using Astra as their codebase operating system.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-3">
            <a href="/signup" className="relative group overflow-hidden bg-white text-gray-900 px-5 md:px-6 py-2.5 md:py-3 text-xs md:text-sm font-medium transition-all duration-300 w-full sm:w-auto hover:bg-gray-100 hover:-translate-y-0.5 hover:shadow-lg">
              <span className="relative z-10 inline-flex items-center gap-2">
                Start building for free
                <ArrowRight className="w-3 h-3 md:w-4 md:h-4 group-hover:translate-x-1 transition-transform duration-300" />
              </span>
            </a>
            <a href="/contact" className="border border-gray-700 text-white px-5 md:px-6 py-2.5 md:py-3 text-xs md:text-sm font-medium transition-all duration-300 w-full sm:w-auto hover:bg-gray-800 hover:-translate-y-0.5 text-center">
              Talk to sales
            </a>
          </div>

          <p className="text-[10px] md:text-xs text-gray-500 mt-4 md:mt-6">
            No credit card required • Free forever for individuals
          </p>
        </motion.div>
      </div>
    </section>
  );
};

export default DarkSection;
