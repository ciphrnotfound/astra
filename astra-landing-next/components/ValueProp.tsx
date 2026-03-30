'use client';

import { motion } from 'framer-motion';

const ValueProp = () => {
  return (
    <section className="py-24 bg-white relative overflow-hidden">
      <div className="max-w-7xl mx-auto px-6">
        <motion.div
           initial={{ opacity: 0, y: 20 }}
           whileInView={{ opacity: 1, y: 0 }}
           viewport={{ once: true }}
           transition={{ duration: 0.5 }}
           className="text-center mb-20"
        >
          <h2 className="text-4xl md:text-5xl font-bold text-black mb-4 tracking-[-0.03em]" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
            One platform.
            <span className="text-[#2F55FF] ml-2">Endless possibilities.</span>
          </h2>
          <p className="text-gray-500 font-medium text-lg">Your codebase, refined and ready.</p>
        </motion.div>

        <div className="grid md:grid-cols-2 gap-10">
          {/* For Developers Card */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="p-12 rounded-[2.5rem] border border-gray-100 bg-[#FAF9F6] flex flex-col justify-between"
          >
            <div>
              <div className="text-[#92400E] font-bold uppercase tracking-widest text-xs mb-6">For Developers</div>
              <h3 className="text-4xl font-bold text-black mb-6 tracking-tight" style={{ fontFamily: 'var(--font-space-grotesk)' }}>Personal</h3>
              <ul className="space-y-4 mb-10">
                {['Local-first inference', 'Persistent CLI history', 'Automated code refactoring', 'Context-aware suggestions'].map((item, i) => (
                  <li key={i} className="flex items-center gap-3 text-gray-600 font-medium">
                    <div className="w-1.5 h-1.5 rounded-full bg-[#92400E]" />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <div className="text-3xl font-bold text-black mb-6">$0 <span className="text-lg text-gray-400 font-normal">/ month</span></div>
              <button className="w-full bg-[#92400E]/10 hover:bg-[#92400E]/20 text-[#92400E] font-bold py-4 rounded-xl transition-all">
                Get started
              </button>
            </div>
          </motion.div>

          {/* For Teams Card */}
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="p-12 rounded-[2.5rem] border border-blue-100 bg-blue-50/30 flex flex-col justify-between"
          >
            <div>
              <div className="text-[#2F55FF] font-bold uppercase tracking-widest text-xs mb-6">For Teams</div>
              <h3 className="text-4xl font-bold text-black mb-6 tracking-tight" style={{ fontFamily: 'var(--font-space-grotesk)' }}>Professional</h3>
              <ul className="space-y-4 mb-10">
                {['Shared team knowledge', 'Global sync across users', 'Advanced role management', 'Priority cloud processing'].map((item, i) => (
                  <li key={i} className="flex items-center gap-3 text-gray-600 font-medium">
                    <div className="w-1.5 h-1.5 rounded-full bg-[#2F55FF]" />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <div className="text-3xl font-bold text-black mb-6">$19 <span className="text-lg text-gray-400 font-normal">/ month per user</span></div>
              <button className="w-full bg-[#2F55FF] hover:bg-blue-700 text-white font-bold py-4 rounded-xl transition-all shadow-lg shadow-blue-200">
                Start 14-day trial
              </button>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
};

export default ValueProp;
