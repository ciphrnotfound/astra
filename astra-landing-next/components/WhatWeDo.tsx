'use client';

import { motion } from 'framer-motion';
import { Code2, Zap, Shield } from 'lucide-react';

const capabilities = [
  {
    icon: Code2,
    title: 'Semantic Understanding',
    description: 'Astra doesn\'t just read syntax—it understands your code\'s meaning, dependencies, and patterns across your entire codebase.',
  },
  {
    icon: Zap,
    title: 'Instant Refactoring',
    description: 'Transform your code in real-time. Rename, restructure, or refactor entire modules while maintaining perfect type safety.',
  },
  {
    icon: Shield,
    title: 'Type-Safe Migrations',
    description: 'Migrate between languages with confidence. Every line is validated against both source and target type systems.',
  },
];

export default function WhatWeDo() {
  return (
    <section className="py-32 px-6 bg-white">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center mb-20"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-gray-200 bg-[#faf9f6] text-xs font-medium text-gray-700 mb-6">
            What we do
          </div>
          
          <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 leading-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            AI-powered code intelligence
            <br />
            <span className="text-gray-600">that actually works</span>
          </h2>
        </motion.div>

        <div className="grid md:grid-cols-3 gap-8">
          {capabilities.map((capability, index) => (
            <motion.div
              key={capability.title}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ duration: 0.6, delay: index * 0.15 }}
              className="group relative"
            >
              <div className="relative bg-[#faf9f6] border border-gray-200 p-8 h-full transition-all duration-300 hover:border-gray-900">
                <div className="mb-6">
                  <div className="w-12 h-12 border border-gray-900 bg-white flex items-center justify-center">
                    <capability.icon className="w-6 h-6 text-gray-900" />
                  </div>
                </div>
                
                <h3 className="text-xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {capability.title}
                </h3>
                
                <p className="text-gray-600 leading-relaxed">
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
