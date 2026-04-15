'use client';

import { motion } from 'framer-motion';
import { Code2, Zap, Shield, GitBranch } from 'lucide-react';

const features = [
  {
    icon: Code2,
    title: 'Semantic Code Graph',
    description: 'Builds a living knowledge graph of your entire codebase. Understand dependencies, trace ownership, and discover hidden coupling instantly.',
  },
  {
    icon: Zap,
    title: 'Persistent Memory',
    description: 'Every conversation, decision, and insight is stored locally and remembered forever. Astra learns your codebase and gets smarter with every commit.',
  },
  {
    icon: Shield,
    title: 'Time Travel Debugging',
    description: 'Use :bisect to find the exact commit that introduced a bug. Step backward through execution history and see every state change.',
  },
  {
    icon: GitBranch,
    title: 'Agent Mode',
    description: 'Autonomous code editing with tool use. Astra can read, write, search, and execute commands to complete complex tasks independently.',
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
          <h2 className="text-2xl sm:text-3xl md:text-4xl font-semibold font-cabinet text-gray-900 mb-3 sm:mb-4">
            Your codebase, permanently understood
          </h2>
          <p className="text-base sm:text-lg text-gray-600 max-w-2xl mx-auto px-4">
            Unlike other tools that forget you the moment you close the terminal, Astra never does.
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
              className="group p-6 sm:p-8 rounded-lg border border-gray-200 hover:border-gray-900 transition-all duration-300 hover:shadow-lg hover:-translate-y-1 bg-white"
            >
              <div className="w-10 h-10 sm:w-12 sm:h-12 rounded-lg bg-gray-100 flex items-center justify-center mb-3 sm:mb-4 group-hover:bg-gray-900 transition-all duration-300 group-hover:scale-110">
                <feature.icon className="w-5 h-5 sm:w-6 sm:h-6 text-gray-900 group-hover:text-white transition-colors duration-300" />
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
