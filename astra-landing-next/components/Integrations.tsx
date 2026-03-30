'use client';

import { motion } from 'framer-motion';

const Integrations = () => {
  const integrationLogos = [
    { name: 'Slack', icon: 'https://cdn.simpleicons.org/slack/4A154B' },
    { name: 'GitHub', icon: 'https://cdn.simpleicons.org/github/181717' },
    { name: 'Discord', icon: 'https://cdn.simpleicons.org/discord/5865F2' },
    { name: 'Notion', icon: 'https://cdn.simpleicons.org/notion/000000' },
    { name: 'Jira', icon: 'https://cdn.simpleicons.org/jira/0052CC' },
    { name: 'Linear', icon: 'https://cdn.simpleicons.org/linear/5E6AD2' },
    { name: 'VS Code', icon: 'https://cdn.simpleicons.org/visualstudiocode/007ACC' },
    { name: 'Vercel', icon: 'https://cdn.simpleicons.org/vercel/000000' },
  ];

  return (
    <section className="py-32 bg-white font-sans overflow-hidden">
      <div className="max-w-7xl mx-auto px-6">
        <div className="text-center mb-16">
          <h2 className="text-4xl font-bold text-black mb-4 tracking-[-0.03em]" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
            Integrate with your flow.
          </h2>
          <p className="text-gray-500 font-medium">Astra works with the tools you already use every day.</p>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
           {integrationLogos.map((logo, i) => (
             <motion.div
               key={i}
               whileHover={{ scale: 1.05 }}
               className="p-10 rounded-[2rem] border border-gray-100 flex items-center justify-center hover:shadow-xl transition-all h-40"
             >
                <img src={logo.icon} alt={logo.name} className="w-12 h-12 grayscale hover:grayscale-0 transition-opacity" />
             </motion.div>
           ))}
        </div>
        
        <div className="mt-16 text-center">
           <button className="text-[#2F55FF] font-bold text-sm tracking-widest uppercase hover:underline">
              View all 50+ integrations
           </button>
        </div>
      </div>
    </section>
  );
};

export default Integrations;
