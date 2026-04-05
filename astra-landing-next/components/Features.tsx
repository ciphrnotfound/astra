'use client';

import { motion } from 'framer-motion';
import { Code2, Zap, Shield, GitBranch } from 'lucide-react';

const features = [
  {
    icon: Code2,
    title: 'Cross-Language Migration',
    description: 'Migrate entire codebases between TypeScript, Rust, Python, Go, Java, and JavaScript. AI-powered translation with semantic cleanup and auto-fix.',
  },
  {
    icon: Zap,
    title: 'Time Travel Debugging',
    description: 'Use :bisect to find the exact commit that introduced a bug. Rewind execution history and replay state changes to debug complex issues.',
  },
  {
    icon: Shield,
    title: 'Security Hunter',
    description: 'Run :security-scan to detect vulnerabilities across your codebase. Proactive security analysis powered by AI pattern recognition.',
  },
  {
    icon: GitBranch,
    title: 'Team OS',
    description: 'Built-in task tracking, time logging, and productivity reports. Implicit session tracking syncs to Supabase for team visibility.',
  },
];

export default function Features() {
  return (
    <section className="py-12 sm:py-16 md:py-24 px-4 sm:px-6 bg-[#faf9f6]">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="text-center mb-12 sm:mb-16"
        >
          <h2 className="text-2xl sm:text-3xl md:text-4xl font-semibold text-gray-900 mb-3 sm:mb-4">
            Built for modern development
          </h2>
          <p className="text-base sm:text-lg text-gray-600 max-w-2xl mx-auto px-4">
            Astra combines AI with deep language understanding to transform how you work with code.
          </p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 sm:gap-6 md:gap-8">
          {features.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              className="group p-6 sm:p-8 rounded-lg border border-gray-200 hover:border-gray-300 transition-all hover:shadow-lg bg-white"
            >
              <div className="w-10 h-10 sm:w-12 sm:h-12 rounded-lg bg-gray-100 flex items-center justify-center mb-3 sm:mb-4 group-hover:bg-gray-900 transition-colors">
                <feature.icon className="w-5 h-5 sm:w-6 sm:h-6 text-gray-900 group-hover:text-white transition-colors" />
              </div>
              <h3 className="text-lg sm:text-xl font-semibold text-gray-900 mb-2">
                {feature.title}
              </h3>
              <p className="text-sm sm:text-base text-gray-600 leading-relaxed">
                {feature.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
