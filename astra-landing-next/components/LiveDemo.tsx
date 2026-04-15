'use client';

import { motion } from 'framer-motion';
import { Terminal, GitBranch, Brain, Zap } from 'lucide-react';

export default function LiveDemo() {
  const commands = [
    { icon: Terminal, cmd: 'astra :index', desc: 'Build semantic graph of your codebase' },
    { icon: GitBranch, cmd: 'astra :why engine.rs', desc: 'Trace complete file history and ownership' },
    { icon: Brain, cmd: 'astra :analyze', desc: 'Extract concepts from git patterns' },
    { icon: Zap, cmd: 'astra --agent "refactor auth"', desc: 'Autonomous code editing with tool use' },
  ];

  return (
    <section className="py-16 md:py-32 px-4 md:px-6 bg-[#faf9f6]">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-12 md:mb-20"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-gray-200 bg-white text-xs font-medium text-gray-700 mb-4 md:mb-6">
            See it in action
          </div>
          
          <h2 className="text-2xl md:text-4xl lg:text-5xl font-medium text-gray-900 mb-4 md:mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            One command.
            <br />
            <span className="text-gray-600">Infinite possibilities.</span>
          </h2>
        </motion.div>

        <div className="grid md:grid-cols-2 gap-6 md:gap-8">
          {commands.map((item, index) => (
            <motion.div
              key={item.cmd}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.1 }}
              className="group"
            >
              <div className="bg-white border border-gray-200 p-6 md:p-8 transition-all duration-300 hover:border-gray-900 hover:-translate-y-1 hover:shadow-lg">
                <div className="w-10 h-10 md:w-12 md:h-12 bg-gray-900 flex items-center justify-center mb-4 md:mb-6 transition-transform duration-300 group-hover:scale-110">
                  <item.icon className="w-5 h-5 md:w-6 md:h-6 text-white" />
                </div>
                
                <div className="font-mono text-xs md:text-sm text-gray-900 mb-3 bg-[#faf9f6] px-3 py-2 border border-gray-200 transition-colors duration-300 group-hover:border-gray-900">
                  $ {item.cmd}
                </div>
                
                <p className="text-sm md:text-base text-gray-600">
                  {item.desc}
                </p>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
