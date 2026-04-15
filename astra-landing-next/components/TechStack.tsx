'use client';

import { motion } from 'framer-motion';
import { Database, Cpu, Lock, Workflow } from 'lucide-react';

export default function TechStack() {
  const stack = [
    {
      icon: Database,
      title: 'Local-First Architecture',
      description: 'All data stored locally in .astra/. Optional cloud sync via Supabase. Your code never leaves your machine.',
      features: ['SQLite storage', 'Git integration', 'Zero latency'],
    },
    {
      icon: Cpu,
      title: 'Rust Performance',
      description: 'Built in Rust for blazing-fast indexing and analysis. Process millions of lines in seconds.',
      features: ['Sub-second search', 'Parallel processing', 'Memory efficient'],
    },
    {
      icon: Lock,
      title: 'Privacy by Design',
      description: 'No telemetry. No tracking. Your codebase stays private. Open source and auditable.',
      features: ['Offline-first', 'Self-hosted', 'MIT licensed'],
    },
    {
      icon: Workflow,
      title: 'Agent Framework',
      description: 'Tool-use loop with autonomous code editing. Reads, writes, searches, and executes commands.',
      features: ['Multi-step reasoning', 'Error recovery', 'Context retention'],
    },
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
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-gray-200 bg-[#faf9f6] text-xs font-medium text-gray-700 mb-4 md:mb-6">
            Architecture
          </div>
          
          <h2 className="text-2xl md:text-4xl lg:text-5xl font-medium text-gray-900 mb-4 md:mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Built for speed.
            <br />
            <span className="text-gray-600">Designed for privacy.</span>
          </h2>
          
          <p className="text-base md:text-lg text-gray-600 max-w-2xl mx-auto">
            Enterprise-grade architecture that runs entirely on your machine.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-2 gap-6 md:gap-8">
          {stack.map((item, index) => (
            <motion.div
              key={item.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              className="group"
            >
              <div className="bg-[#faf9f6] border border-gray-200 p-6 md:p-8 h-full transition-all duration-300 hover:border-gray-900 hover:-translate-y-1">
                <div className="w-10 h-10 md:w-12 md:h-12 bg-gray-900 flex items-center justify-center mb-4 md:mb-6 transition-transform duration-300 group-hover:scale-110">
                  <item.icon className="w-5 h-5 md:w-6 md:h-6 text-white" />
                </div>
                
                <h3 className="text-lg md:text-xl font-medium text-gray-900 mb-3 md:mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {item.title}
                </h3>
                
                <p className="text-sm md:text-base text-gray-600 mb-4 md:mb-6 leading-relaxed">
                  {item.description}
                </p>
                
                <ul className="space-y-2">
                  {item.features.map((feature) => (
                    <li key={feature} className="flex items-center gap-2 text-xs md:text-sm text-gray-700">
                      <div className="w-1.5 h-1.5 bg-gray-900" />
                      {feature}
                    </li>
                  ))}
                </ul>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
