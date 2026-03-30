'use client';

import { motion } from 'framer-motion';
import { Code2, Zap, Shield, GitBranch } from 'lucide-react';

const features = [
  {
    icon: Code2,
    title: 'Cross-Language Migration',
    description: 'Seamlessly migrate code between TypeScript, Rust, Python, Go, and more. Astra understands your code semantically, not just syntactically.',
  },
  {
    icon: Zap,
    title: 'AI-Powered Refactoring',
    description: 'Intelligent refactoring that preserves behavior while improving code quality. Let AI handle the tedious work.',
  },
  {
    icon: Shield,
    title: 'Type-Safe Transformations',
    description: 'Every migration is validated for type safety and correctness. No runtime surprises, guaranteed.',
  },
  {
    icon: GitBranch,
    title: 'Time Travel Debugging',
    description: 'Step through your code execution history. Debug complex issues by rewinding and replaying state changes.',
  },
];

export default function Features() {
  return (
    <section className="py-24 px-6 bg-[#faf9f6]">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="text-center mb-16"
        >
          <h2 className="text-3xl md:text-4xl font-semibold text-gray-900 mb-4">
            Built for modern development
          </h2>
          <p className="text-lg text-gray-600 max-w-2xl mx-auto">
            Astra combines AI with deep language understanding to transform how you work with code.
          </p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          {features.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              className="group p-8 rounded-lg border border-gray-200 hover:border-gray-300 transition-all hover:shadow-lg"
            >
              <div className="w-12 h-12 rounded-lg bg-gray-100 flex items-center justify-center mb-4 group-hover:bg-gray-900 transition-colors">
                <feature.icon className="w-6 h-6 text-gray-900 group-hover:text-white transition-colors" />
              </div>
              <h3 className="text-xl font-semibold text-gray-900 mb-2">
                {feature.title}
              </h3>
              <p className="text-gray-600 leading-relaxed">
                {feature.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
