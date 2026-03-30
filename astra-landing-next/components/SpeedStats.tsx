'use client';

import { motion } from 'framer-motion';
import { Play } from 'lucide-react';

const SpeedStats = () => {
  return (
    <section className="py-32 bg-white overflow-hidden font-sans">
      <div className="max-w-7xl mx-auto px-6">
        <div className="text-center mb-16">
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="text-5xl font-bold text-black mb-6 tracking-[-0.03em]"
            style={{ fontFamily: 'var(--font-space-grotesk)' }}
          >
            Built for speed.
            <span className="text-gray-400 block mt-2">Optimized for scale.</span>
          </motion.h2>
          <p className="text-gray-500 font-medium text-lg max-w-2xl mx-auto">
            Astra is designed to handle the most demanding development workflows with sub-second retrieval times.
          </p>
        </div>

        {/* Video Player Mockup */}
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          whileInView={{ opacity: 1, scale: 1 }}
          viewport={{ once: true }}
          className="relative aspect-video max-w-5xl mx-auto rounded-[2.5rem] bg-black overflow-hidden shadow-2xl mb-16 border-8 border-gray-100/50"
        >
          <div className="absolute inset-0 bg-[#0A0C10] flex items-center justify-center">
             <div className="text-center">
                <div className="text-[10vw] font-black text-white/5 opacity-20 select-none mb-[-5%]">STATE OF</div>
                <div className="text-[12vw] font-black text-[#2F55FF] tracking-tighter shadow-blue-500/20">THE ART</div>
             </div>
             
             {/* Player UI */}
             <div className="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity">
                <div className="w-24 h-24 bg-[#2F55FF] rounded-full flex items-center justify-center text-white shadow-2xl scale-0 group-hover:scale-100 transition-transform">
                  <Play className="w-10 h-10 fill-current" />
                </div>
             </div>
          </div>
          
          {/* Bottom Bar */}
          <div className="absolute bottom-6 left-6 right-6 flex items-center justify-between text-white/60 text-xs font-bold uppercase tracking-widest">
             <div className="flex items-center gap-4">
                <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
                LIVE DEMO: v2.0
             </div>
             <div>HD 1080P</div>
          </div>
        </motion.div>

        {/* Metrics Row */}
        <div className="grid grid-cols-3 gap-12 max-w-4xl mx-auto text-center mb-24">
           {[
             { label: 'Recovery Time', value: '2.4s' },
             { label: 'Search Precision', value: '98%' },
             { label: 'Availability', value: '24/7' },
           ].map((stat, i) => (
             <div key={i}>
                <div className="text-4xl font-bold text-black mb-1" style={{ fontFamily: 'var(--font-space-grotesk)' }}>{stat.value}</div>
                <div className="text-[11px] font-bold text-gray-400 uppercase tracking-widest">{stat.label}</div>
             </div>
           ))}
        </div>

        {/* Highlight Grid */}
        <div className="grid md:grid-cols-3 gap-8">
           {[
             { title: 'Real-time sync', desc: 'Instant updates across all sessions.' },
             { title: 'Advanced search', desc: 'Hybrid retrieval with semantic depth.' },
             { title: 'High-performance', desc: 'Optimized for massive codebases.' },
           ].map((item, i) => (
             <motion.div
               key={i}
               whileHover={{ y: -5 }}
               className="p-8 rounded-[2rem] border border-gray-100 bg-gray-50/30 text-center"
             >
                <div className="w-1.5 h-1.5 bg-[#2F55FF] rounded-full mx-auto mb-4" />
                <h4 className="text-lg font-bold text-black mb-2 tracking-tight" style={{ fontFamily: 'var(--font-space-grotesk)' }}>{item.title}</h4>
                <p className="text-gray-500 font-medium text-sm">{item.desc}</p>
             </motion.div>
           ))}
        </div>
      </div>
    </section>
  );
};

export default SpeedStats;
