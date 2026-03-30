'use client';

import { motion } from 'framer-motion';
import { Terminal, Download, Rocket } from 'lucide-react';

const Setup = () => {
  const steps = [
    {
      icon: Download,
      title: 'Install',
      command: 'cargo install astra-cli',
      description: 'One command to get started',
    },
    {
      icon: Terminal,
      title: 'Initialize',
      command: 'astra init',
      description: 'Set up your project in seconds',
    },
    {
      icon: Rocket,
      title: 'Start Building',
      command: 'astra migrate typescript rust',
      description: 'Begin your first migration',
    },
  ];

  return (
    <section className="py-32 bg-[#faf9f6] px-6">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8 }}
          className="text-center mb-20"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-gray-200 bg-white text-xs font-medium text-gray-700 mb-6">
            Get started
          </div>
          
          <h2 className="text-4xl md:text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Up and running
            <br />
            <span className="text-gray-600">in under 60 seconds</span>
          </h2>
        </motion.div>

        <div className="grid md:grid-cols-3 gap-6">
          {steps.map((step, index) => (
            <motion.div
              key={index}
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: index * 0.15 }}
              className="relative"
            >
              <div className="bg-white border border-gray-200 p-8 h-full transition-all duration-300 hover:border-gray-900">
                <div className="flex items-center gap-3 mb-6">
                  <div className="w-10 h-10 border border-gray-900 bg-white flex items-center justify-center">
                    <step.icon className="w-5 h-5 text-gray-900" />
                  </div>
                  <span className="text-sm text-gray-500">Step {index + 1}</span>
                </div>
                
                <h3 className="text-xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {step.title}
                </h3>
                
                <div className="bg-gray-900 p-4 mb-4 font-mono text-sm text-green-400">
                  $ {step.command}
                </div>
                
                <p className="text-gray-600 text-sm">
                  {step.description}
                </p>
              </div>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.6 }}
          className="mt-12 text-center"
        >
          <button className="relative group overflow-hidden bg-gray-900 text-white px-8 py-4 text-sm font-medium transition-all hover:shadow-lg">
            <span className="relative z-10">View full documentation →</span>
            <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
          </button>
        </motion.div>
      </div>
    </section>
  );
};

export default Setup;
