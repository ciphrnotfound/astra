'use client';

import { useRef } from 'react';
import { motion, useInView } from 'framer-motion';
import { Database, GitBranch, Code, Cpu, Sparkles } from 'lucide-react';

const FiveLayers = () => {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  const layers = [
    { title: 'Semantic Search', desc: 'Understanding intent, not just keywords.' },
    { title: 'Knowledge Graph', desc: 'Contextual relationships at scale.' },
    { title: 'Vector Database', desc: 'High-dimensional similarity retrieval.' },
    { title: 'Context Window', desc: 'Optimized long-term memory access.' },
    { title: 'Memory Management', desc: 'Intelligent pruning and focus.' },
  ];

  return (
    <section ref={ref} className="py-32 bg-gray-50/30 overflow-hidden">
      <div className="max-w-7xl mx-auto px-6">
        <div className="grid lg:grid-cols-2 gap-20 items-center">
          {/* Left: 3D Glass Stack Illustration */}
          <div className="relative h-[600px] flex items-center justify-center perspective-[1000px]">
            {layers.map((_, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, y: 100, rotateX: 45, rotateZ: -10 }}
                animate={isInView ? { 
                  opacity: 1, 
                  y: -i * 50,
                  rotateX: 45,
                  rotateZ: -10,
                } : {}}
                transition={{ duration: 0.8, delay: i * 0.1, ease: [0.23, 1, 0.32, 1] }}
                className="absolute w-[350px] aspect-[16/10] bg-blue-500/10 border-2 border-white/40 rounded-2xl shadow-2xl backdrop-blur-md"
                style={{ transformStyle: 'preserve-3d' }}
              >
                <div className="absolute inset-0 bg-gradient-to-br from-white/20 to-transparent rounded-2xl" />
                {i === 4 && (
                  <div className="absolute inset-0 flex items-center justify-center rotate-[-45deg] scale-[1.5]">
                    <div className="w-12 h-12 bg-white rounded-lg shadow-xl" />
                  </div>
                )}
              </motion.div>
            ))}
          </div>

          {/* Right: Detailed List */}
          <div className="space-y-12">
            <motion.div
              initial={{ opacity: 0, x: 30 }}
              animate={isInView ? { opacity: 1, x: 0 } : {}}
              transition={{ duration: 0.6 }}
            >
              <h2 className="text-5xl font-bold text-black leading-tight mb-4 tracking-[-0.03em]" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
                The five layers of
                <br />
                the Astra engine.
              </h2>
              <p className="text-lg text-gray-500 font-medium">
                A multi-layered architecture for reliable AI execution.
              </p>
            </motion.div>

            <div className="space-y-8">
              {layers.map((layer, i) => (
                <motion.div 
                  key={i}
                  initial={{ opacity: 0, x: 20 }}
                  animate={isInView ? { opacity: 1, x: 0 } : {}}
                  transition={{ delay: 0.3 + i * 0.1 }}
                  className="flex gap-6 group"
                >
                  <div className="w-10 h-10 rounded-full bg-blue-100 flex items-center justify-center text-[#2F55FF] font-bold text-sm shrink-0 group-hover:bg-[#2F55FF] group-hover:text-white transition-all">
                    {i + 1}
                  </div>
                  <div>
                    <h4 className="text-xl font-bold text-black mb-1 tracking-tight" style={{ fontFamily: 'var(--font-space-grotesk)' }}>{layer.title}</h4>
                    <p className="text-gray-500 font-medium leading-relaxed">{layer.desc}</p>
                  </div>
                </motion.div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default FiveLayers;
