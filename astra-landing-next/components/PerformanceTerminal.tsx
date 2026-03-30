'use client';

import { motion } from 'framer-motion';

const PerformanceTerminal = () => {
  return (
    <section className="py-32 bg-white font-sans overflow-hidden">
      <div className="max-w-7xl mx-auto px-6">
        <div className="text-center mb-16">
          <h2 className="text-5xl font-bold text-black mb-6 tracking-[-0.03em]" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
            Design for performance.
          </h2>
          <p className="text-gray-500 font-medium text-lg max-w-2xl mx-auto">
            Astra is built for developer productivity, providing a seamless DX with powerful CLI and SDK capabilities.
          </p>
        </div>

        <div className="max-w-4xl mx-auto">
           {/* Code Editor Mockup */}
           <motion.div
             initial={{ opacity: 0, y: 20 }}
             whileInView={{ opacity: 1, y: 0 }}
             viewport={{ once: true }}
             className="bg-[#0A0C10] rounded-t-[2rem] border-x border-t border-white/10 overflow-hidden shadow-2xl"
           >
              <div className="flex items-center gap-2 px-6 py-4 border-b border-white/5 bg-white/5">
                 <div className="w-3 h-3 rounded-full bg-red-400/50" />
                 <div className="w-3 h-3 rounded-full bg-yellow-400/50" />
                 <div className="w-3 h-3 rounded-full bg-green-400/50" />
                 <div className="ml-4 text-xs font-bold text-gray-500 uppercase tracking-widest font-sans">astra_search.ts</div>
              </div>
              <div className="p-8 font-mono text-sm leading-relaxed">
                 <div className="text-blue-400">import <span className="text-white">{`{ Astra }`}</span> from <span className="text-emerald-400">"astra-core"</span>;</div>
                 <div className="h-4" />
                 <div className="text-gray-500">{`// Initialize Astra with local-first preference`}</div>
                 <div><span className="text-purple-400">const</span> <span className="text-white">astra</span> = <span className="text-purple-400">new</span> <span className="text-blue-400">Astra</span>({`{`}</div>
                 <div className="pl-6">apiKey: <span className="text-white">process.env.ASTRA_API_KEY</span>,</div>
                 <div className="pl-6">localFirst: <span className="text-emerald-400">true</span></div>
                 <div>{`});`}</div>
                 <div className="h-4" />
                 <div className="text-gray-500">{`// Semantic search across entire history`}</div>
                 <div><span className="text-purple-400">const</span> <span className="text-white">context</span> = <span className="text-purple-400">await</span> <span className="text-white">astra.search(</span><span className="text-emerald-400">"login fails on mobile"</span><span className="text-white">);</span></div>
              </div>
           </motion.div>

           {/* CLI Mockup */}
           <motion.div
             initial={{ opacity: 0, y: 10 }}
             whileInView={{ opacity: 1, y: 0 }}
             viewport={{ once: true }}
             transition={{ delay: 0.2 }}
             className="bg-black/95 rounded-b-[2rem] border border-white/10 overflow-hidden shadow-2xl mt-[-1px]"
           >
              <div className="p-8 font-mono text-[13px] leading-relaxed">
                 <div className="flex gap-3">
                    <span className="text-emerald-400">$</span>
                    <span className="text-white">astra --bisect "login fails on mobile"</span>
                 </div>
                 <div className="text-gray-500 mt-2">[ASTRA] Analyzing 42 commits from last 7 days...</div>
                 <div className="text-gray-500">[ASTRA] Running semantic correlation check...</div>
                 <div className="text-[#2F55FF] font-bold mt-2">[ASTRA] Found potential cause in commit 8a2b1c (auth_provider.ts)</div>
                 <div className="text-white mt-1">{`> Match: "Mobile authentication bridge initialization omitted"`}</div>
                 <div className="text-emerald-400 font-bold mt-2">✓ 98% confidence score. Result saved to .astra/memory</div>
              </div>
           </motion.div>
        </div>
      </div>
    </section>
  );
};

export default PerformanceTerminal;
