'use client';

import { motion } from 'framer-motion';

const steps = [
  {
    number: '01',
    title: 'Initialize Astra',
    command: 'astra init',
    description: 'Initialize Astra in your project directory',
  },
  {
    number: '02',
    title: 'Index your codebase',
    command: 'astra index',
    description: 'Build semantic graph of your entire codebase',
  },
  {
    number: '03',
    title: 'Start exploring',
    command: 'astra "what does this project do?"',
    description: 'Ask questions about your code in natural language',
  },
];

export default function QuickStart() {
  return (
    <section className="py-32 px-6 bg-[#faf9f6]">
      <div className="max-w-4xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
        >
          <h2 className="text-4xl font-medium text-gray-900 mb-12 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Quick Start
          </h2>

          <div className="space-y-8">
            {steps.map((step, index) => (
              <motion.div
                key={step.number}
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
                className="flex gap-6"
              >
                <div className="text-4xl font-bold text-gray-200" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {step.number}
                </div>
                <div className="flex-1">
                  <h3 className="text-xl font-medium text-gray-900 mb-2">
                    {step.title}
                  </h3>
                  <pre className="bg-gray-900 text-gray-100 p-4 mb-3 text-sm font-mono overflow-x-auto">
                    $ {step.command}
                  </pre>
                  <p className="text-gray-600">
                    {step.description}
                  </p>
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  );
}
