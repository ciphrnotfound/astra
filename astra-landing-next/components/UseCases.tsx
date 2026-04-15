'use client';

import { motion } from 'framer-motion';
import { ArrowRight, Users, Code, Building2 } from 'lucide-react';

const useCases = [
  {
    icon: Users,
    tag: 'For Teams',
    title: 'Onboard developers instantly',
    description: 'New team members ask Astra "why did we build it this way?" and get answers from your git history, not outdated docs.',
    metrics: [
      { label: 'Onboarding time', value: '10x faster' },
      { label: 'Context retention', value: '100%' },
    ],
  },
  {
    icon: Code,
    tag: 'For Developers',
    title: 'Debug with time travel',
    description: 'Step backward through your code execution. See every state change, every function call, every decision your code made.',
    metrics: [
      { label: 'Debug speed', value: '5x faster' },
      { label: 'Bug detection', value: '95%' },
    ],
  },
  {
    icon: Building2,
    tag: 'For Enterprises',
    title: 'Track codebase health',
    description: 'Real-time health scores for code quality, security surface, test coverage, and team velocity. Prevent technical debt before it happens.',
    metrics: [
      { label: 'Health visibility', value: '100%' },
      { label: 'Debt prevention', value: '80%' },
    ],
  },
];

export default function UseCases() {
  return (
    <section className="py-16 md:py-32 px-4 md:px-6 bg-white">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8, ease: 'easeOut' }}
          className="text-center mb-12 md:mb-20"
        >
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-gray-200 bg-[#faf9f6] text-xs font-medium text-gray-700 mb-4 md:mb-6">
            Use cases
          </div>
          
          <h2 className="text-2xl md:text-4xl lg:text-5xl font-medium text-gray-900 mb-4 md:mb-6 leading-tight px-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Built for every stage
            <br />
            <span className="text-gray-600">of your development journey</span>
          </h2>
        </motion.div>

        <div className="grid md:grid-cols-3 gap-6 md:gap-8">
          {useCases.map((useCase, index) => (
            <motion.div
              key={useCase.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ 
                duration: 0.5,
                delay: index * 0.1,
                ease: 'easeOut',
              }}
              className="group relative"
            >
              <div className="relative bg-[#faf9f6] border border-gray-200 p-6 md:p-8 h-full transition-all duration-300 hover:border-gray-900 hover:shadow-lg hover:-translate-y-1">
                <div className="mb-4 md:mb-6">
                  <div className="w-10 h-10 md:w-12 md:h-12 border border-gray-900 bg-white flex items-center justify-center mb-3 transition-transform duration-300 group-hover:scale-110">
                    <useCase.icon className="w-5 h-5 md:w-6 md:h-6 text-gray-900" />
                  </div>
                  <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">
                    {useCase.tag}
                  </span>
                </div>
                
                <h3 className="text-xl md:text-2xl font-medium text-gray-900 mb-3 md:mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {useCase.title}
                </h3>
                
                <p className="text-sm md:text-base text-gray-600 leading-relaxed mb-6">
                  {useCase.description}
                </p>

                <div className="space-y-3 pt-4 border-t border-gray-200">
                  {useCase.metrics.map((metric) => (
                    <div key={metric.label} className="flex items-center justify-between">
                      <span className="text-xs text-gray-500 uppercase tracking-wide">{metric.label}</span>
                      <span className="text-sm font-medium text-gray-900">{metric.value}</span>
                    </div>
                  ))}
                </div>
              </div>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.4 }}
          className="mt-12 md:mt-16 text-center"
        >
          <p className="text-sm md:text-base text-gray-600 mb-4 md:mb-6 px-4">
            Ready to give your codebase a memory?
          </p>
          <button className="relative group overflow-hidden bg-gray-900 text-white px-6 md:px-8 py-3 md:py-4 text-xs md:text-sm font-medium transition-all hover:shadow-lg">
            <span className="relative z-10">Get started for free</span>
            <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
          </button>
        </motion.div>
      </div>
    </section>
  );
}
