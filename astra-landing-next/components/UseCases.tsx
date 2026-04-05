'use client';

import { motion } from 'framer-motion';
import { ArrowRight, Users, Code, Building2 } from 'lucide-react';

const useCases = [
  {
    icon: Users,
    tag: 'For Teams',
    title: 'Modernize legacy codebases',
    description: 'Migrate from JavaScript to TypeScript, or TypeScript to Rust. Astra handles the complexity while your team stays productive.',
    metrics: [
      { label: 'Migration time', value: '10x faster' },
      { label: 'Code accuracy', value: '99.9%' },
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
    title: 'Scale with confidence',
    description: 'Refactor entire microservices architectures. Astra validates every change across your entire codebase before deployment.',
    metrics: [
      { label: 'Services migrated', value: '100+' },
      { label: 'Zero downtime', value: '100%' },
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

        <div className="space-y-4 md:space-y-8">
          {useCases.map((useCase, index) => (
            <motion.div
              key={useCase.title}
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true, margin: '-100px' }}
              transition={{ 
                duration: 0.6,
                delay: index * 0.15,
                ease: 'easeOut',
              }}
              className="group relative"
            >
              <div className="relative bg-[#faf9f6] border border-gray-200 p-6 md:p-8 lg:p-10 transition-all duration-300 hover:border-gray-900">
                <div className="grid md:grid-cols-[1fr,auto] gap-6 md:gap-8 items-center">
                  <div>
                    <div className="inline-flex items-center gap-2 mb-3 md:mb-4">
                      <div className="w-7 h-7 md:w-8 md:h-8 border border-gray-900 bg-white flex items-center justify-center">
                        <useCase.icon className="w-3.5 h-3.5 md:w-4 md:h-4 text-gray-900" />
                      </div>
                      <span className="text-xs font-medium text-gray-700">
                        {useCase.tag}
                      </span>
                    </div>
                    
                    <h3 className="text-xl md:text-2xl lg:text-3xl font-medium text-gray-900 mb-3 md:mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                      {useCase.title}
                    </h3>
                    
                    <p className="text-sm md:text-base text-gray-600 leading-relaxed mb-4 md:mb-6 max-w-xl">
                      {useCase.description}
                    </p>

                    <button className="inline-flex items-center gap-2 text-xs md:text-sm text-gray-900 font-medium group/btn">
                      <span>Learn more</span>
                      <ArrowRight className="w-3 h-3 md:w-4 md:h-4 transition-transform group-hover/btn:translate-x-1" />
                    </button>
                  </div>

                  <div className="flex flex-row md:flex-col gap-3 md:gap-4">
                    {useCase.metrics.map((metric) => (
                      <div
                        key={metric.label}
                        className="bg-white border border-gray-200 p-4 md:p-6 min-w-[120px] md:min-w-[140px] transition-all duration-300 hover:border-gray-900"
                      >
                        <div
                          className="text-2xl md:text-3xl font-medium text-gray-900 mb-1 md:mb-2"
                          style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}
                        >
                          {metric.value}
                        </div>
                        <div className="text-[10px] md:text-xs text-gray-500 uppercase tracking-wide">
                          {metric.label}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.6 }}
          className="mt-12 md:mt-16 text-center"
        >
          <p className="text-sm md:text-base text-gray-600 mb-4 md:mb-6 px-4">
            Ready to transform your codebase?
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
