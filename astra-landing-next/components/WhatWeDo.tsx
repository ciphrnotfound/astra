'use client';

import { motion } from 'framer-motion';
import { Code2, Zap, Shield } from 'lucide-react';

const capabilities = [
  {
    icon: Code2,
    title: 'Semantic Code Graph',
    description: 'Astra builds a living knowledge graph of your entire codebase. Understand dependencies, trace ownership, and discover hidden coupling instantly.',
  },
  {
    icon: Zap,
    title: 'Persistent Memory',
    description: 'Every conversation, decision, and insight is remembered forever. Astra learns your codebase and gets smarter with every commit.',
  },
  {
    icon: Shield,
    title: 'Agent Mode',
    description: 'Autonomous code editing with tool use. Astra can read, write, search, and execute commands to complete complex tasks independently.',
  },
];

export default function WhatWeDo() {
  return (
    <section className="py-16 md:py-32 px-4 md:px-6 bg-white">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center mb-12 md:mb-20"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-gray-200 bg-[#faf9f6] text-xs font-medium text-gray-700 mb-4 md:mb-6">
            What we do
          </div>
          
          <h2 className="text-2xl md:text-4xl lg:text-5xl font-medium text-gray-900 mb-4 md:mb-6 leading-tight px-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            The codebase operating system
            <br />
            <span className="text-gray-600">that never forgets</span>
          </h2>
        </motion.div>

        <div className="grid md:grid-cols-3 gap-4 md:gap-8">
          {capabilities.map((capability, index) => (
            <motion.div
              key={capability.title}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ duration: 0.6, delay: index * 0.15 }}
              className="group relative"
            >
              <div className="relative bg-[#faf9f6] border border-gray-200 p-6 md:p-8 h-full transition-all duration-300 hover:border-gray-900 hover:-translate-y-1 hover:shadow-lg">
                <div className="mb-4 md:mb-6">
                  <div className="w-10 h-10 md:w-12 md:h-12 border border-gray-900 bg-white flex items-center justify-center transition-transform duration-300 group-hover:scale-110">
                    <capability.icon className="w-5 h-5 md:w-6 md:h-6 text-gray-900" />
                  </div>
                </div>
                
                <h3 className="text-lg md:text-xl font-medium text-gray-900 mb-3 md:mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {capability.title}
                </h3>
                
                <p className="text-sm md:text-base text-gray-600 leading-relaxed">
                  {capability.description}
                </p>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
