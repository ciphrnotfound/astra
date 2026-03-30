'use client';

import { useRef } from 'react';
import { motion, useInView } from 'framer-motion';
import { TrendingUp, Clock, Users, Target } from 'lucide-react';

const KnowledgeEvolves = () => {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  const integrations = [
    { name: 'Slack', icon: 'https://cdn.simpleicons.org/slack/4A154B' },
    { name: 'GitHub', icon: 'https://cdn.simpleicons.org/github/181717' },
    { name: 'Discord', icon: 'https://cdn.simpleicons.org/discord/5865F2' },
    { name: 'Notion', icon: 'https://cdn.simpleicons.org/notion/000000' },
    { name: 'Jira', icon: 'https://cdn.simpleicons.org/jira/0052CC' },
    { name: 'Linear', icon: 'https://cdn.simpleicons.org/linear/5E6AD2' },
  ];

  return (
    <section ref={ref} className="py-32 bg-white relative overflow-hidden">
      <div className="max-w-7xl mx-auto px-6">
        <div className="text-center mb-16">
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            animate={isInView ? { opacity: 1, y: 0 } : {}}
            transition={{ duration: 0.5 }}
            className="text-5xl font-bold text-black tracking-[-0.03em]"
            style={{ fontFamily: 'var(--font-space-grotesk)' }}
          >
            Knowledge evolves
            <span className="text-gray-400 block mt-2">with every action.</span>
          </motion.h2>
        </div>

        {/* Dynamic Network Map */}
        <div className="relative h-[600px] mb-20">
          <div className="absolute inset-0 flex items-center justify-center">
             {/* Central Brain/Head */}
             <motion.div
               animate={{ scale: [1, 1.05, 1] }}
               transition={{ duration: 4, repeat: Infinity }}
               className="w-32 h-32 bg-[#2F55FF] rounded-3xl rotate-45 flex items-center justify-center shadow-2xl shadow-blue-500/20 z-10"
             >
               <div className="rotate-[-45deg] text-white font-black text-4xl">A</div>
             </motion.div>

             {/* Connecting Lines and Icons */}
             {integrations.map((item, i) => {
               const angle = (i * 360) / integrations.length;
               const radius = 220;
               const x = Math.cos((angle * Math.PI) / 180) * radius;
               const y = Math.sin((angle * Math.PI) / 180) * radius;

               return (
                 <div key={i} className="absolute">
                   {/* Line to center */}
                   <motion.div
                     initial={{ scaleX: 0 }}
                     animate={isInView ? { scaleX: 1 } : {}}
                     transition={{ duration: 1, delay: 0.5 + i * 0.1 }}
                     className="absolute w-[220px] h-px bg-gradient-to-r from-[#2F55FF]/40 to-transparent origin-left"
                     style={{ transform: `rotate(${angle}deg)` }}
                   />
                   
                   {/* Icon Bubble */}
                   <motion.div
                     initial={{ opacity: 0, scale: 0 }}
                     animate={isInView ? { opacity: 1, scale: 1 } : {}}
                     transition={{ duration: 0.5, delay: 1 + i * 0.1 }}
                     className="absolute w-14 h-14 bg-white border border-gray-100 rounded-2xl shadow-xl flex items-center justify-center -translate-x-7 -translate-y-7 z-20"
                     style={{ left: x, top: y }}
                   >
                     <img src={item.icon} alt={item.name} className="w-6 h-6 object-contain" />
                   </motion.div>
                 </div>
               );
             })}
          </div>
        </div>

        <div className="grid md:grid-cols-2 gap-12 max-w-4xl mx-auto">
          {[
            { title: 'Pattern Recognition', desc: 'Identifying recurring structures and architectural choices.' },
            { title: 'Contextual Memory', desc: 'Retaining deep knowledge across sessions and projects.' }
          ].map((item, i) => (
            <div key={i} className="flex gap-6">
              <div className="w-1.5 h-auto bg-blue-100 rounded-full" />
              <div>
                <h4 className="text-xl font-bold text-black mb-2 tracking-tight" style={{ fontFamily: 'var(--font-space-grotesk)' }}>{item.title}</h4>
                <p className="text-gray-500 font-medium leading-relaxed">{item.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
       

export default KnowledgeEvolves;
