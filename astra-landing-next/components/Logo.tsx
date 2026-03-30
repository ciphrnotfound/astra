'use client';

import { motion } from 'framer-motion';

const Logo = () => {
  return (
    <motion.div 
      className="flex items-center space-x-2"
      whileHover={{ scale: 1.05 }}
      transition={{ duration: 0.2 }}
    >
      <svg width="40" height="40" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
        {/* White triangle (back layer) */}
        <motion.path
          d="M20 5 L35 32 L5 32 Z"
          fill="white"
          stroke="#080d1a"
          strokeWidth="1.5"
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
        />
        {/* Blue triangle (front layer) - offset for 3D effect */}
        <motion.path
          d="M20 8 L32 30 L8 30 Z"
          fill="#2B8EF0"
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        />
      </svg>
      <span className="text-2xl font-bold" style={{ fontFamily: 'var(--font-syne)' }}>Astra</span>
    </motion.div>
  );
};

export default Logo;
