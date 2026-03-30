'use client';

import { motion } from 'framer-motion';

const commands = [
  {
    category: 'Basic Commands',
    items: [
      { cmd: 'astra init', desc: 'Initialize Astra in your project' },
      { cmd: 'astra index', desc: 'Build semantic graph of codebase' },
      { cmd: 'astra summary', desc: 'Get project overview' },
      { cmd: 'astra health', desc: 'Check codebase health' },
    ],
  },
  {
    category: 'Migration',
    items: [
      { cmd: 'astra migrate <file> from <lang> to <lang>', desc: 'Migrate code between languages' },
      { cmd: 'astra "migrate auth.py to TypeScript"', desc: 'Natural language migration' },
    ],
  },
  {
    category: 'Analysis',
    items: [
      { cmd: 'astra "find all API endpoints"', desc: 'Search codebase semantically' },
      { cmd: 'astra "what depends on User model?"', desc: 'Analyze dependencies' },
      { cmd: 'astra "scan for security issues"', desc: 'Security vulnerability scan' },
    ],
  },
];

export default function Commands() {
  return (
    <section className="py-32 px-6 bg-white">
      <div className="max-w-4xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
        >
          <h2 className="text-4xl font-medium text-gray-900 mb-12 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Command Reference
          </h2>

          <div className="space-y-12">
            {commands.map((section, index) => (
              <div key={section.category}>
                <h3 className="text-2xl font-medium text-gray-900 mb-6" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {section.category}
                </h3>
                <div className="space-y-4">
                  {section.items.map((item, i) => (
                    <motion.div
                      key={i}
                      initial={{ opacity: 0, y: 10 }}
                      whileInView={{ opacity: 1, y: 0 }}
                      viewport={{ once: true }}
                      transition={{ duration: 0.4, delay: i * 0.05 }}
                      className="p-4 border border-gray-200 bg-gray-50"
                    >
                      <code className="text-sm font-mono text-gray-900 block mb-2">
                        $ {item.cmd}
                      </code>
                      <p className="text-sm text-gray-600">
                        {item.desc}
                      </p>
                    </motion.div>
                  ))}
                </div>
              </div>
            ))}
          </div>

          <div className="mt-16 p-8 border border-gray-200 bg-gray-50">
            <h3 className="text-xl font-medium text-gray-900 mb-4">
              Need more help?
            </h3>
            <p className="text-gray-600 mb-6">
              Check out the full documentation on GitHub or join our community.
            </p>
            <div className="flex gap-3">
              <a
                href="https://github.com/yourusername/astra"
                className="relative group overflow-hidden bg-gray-900 text-white px-6 py-2.5 text-sm font-medium transition-all hover:shadow-lg"
              >
                <span className="relative z-10">View on GitHub</span>
                <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
              </a>
              <a
                href="/contact"
                className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-2.5 text-sm font-medium transition-all hover:text-white"
              >
                <span className="relative z-10">Contact us</span>
                <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
              </a>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
