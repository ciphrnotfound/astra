'use client';

import { motion } from 'framer-motion';

const BlueStatsBanner = () => {
  const stats = [
    { label: 'Total Developers', value: '100k+' },
    { label: 'Scale-ready Apps', value: '9k+' },
    { label: 'Uptime', value: '99.9%' },
    { label: 'Live Support', value: '24/7' },
  ];

  return (
    <div className="bg-[#2F55FF] py-12 px-6 overflow-hidden relative">
      <div className="max-w-7xl mx-auto flex flex-wrap justify-between items-center gap-10 relative z-10">
        {stats.map((stat, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: i * 0.1 }}
            className="text-center"
          >
            <div className="text-4xl font-bold text-white mb-1" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
              {stat.value}
            </div>
            <div className="text-blue-100 text-xs font-bold uppercase tracking-widest font-sans">
              {stat.label}
            </div>
          </motion.div>
        ))}
      </div>
      
      {/* Decorative Astra Logo/Star in background */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 opacity-10 pointer-events-none">
        <div className="w-[500px] h-[500px] border border-white rounded-full flex items-center justify-center">
          <div className="w-[300px] h-[300px] border border-white rounded-full rotate-45 flex items-center justify-center">
            <div className="w-20 h-20 bg-white rounded-sm rotate-45" />
          </div>
        </div>
      </div>
    </div>
  );
};

export default BlueStatsBanner;
