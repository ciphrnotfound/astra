'use client';

import { motion, useScroll, useTransform } from 'framer-motion';
import { useRef } from 'react';

export default function GiantAstra() {
  const ref = useRef(null);
  
  // Track scroll progress of this section
  const { scrollYProgress } = useScroll({
    target: ref,
    offset: ["start end", "end start"]
  });

  // Transform scroll progress to fill animation (0 to 1200)
  // Earlier range to ensure it fills on mobile
  const fillX = useTransform(scrollYProgress, [0.1, 0.5], [-1200, 0]);

  return (
    <section ref={ref} className="relative py-24 sm:py-32 md:py-40 overflow-hidden bg-[#faf9f6]">
      <div className="max-w-7xl mx-auto px-4">
        {/* Giant ASTRA text with scroll-based fill animation */}
        <div className="relative">
          <svg
            viewBox="0 0 1200 300"
            className="w-full h-auto"
            xmlns="http://www.w3.org/2000/svg"
          >
            <defs>
              {/* Clip path for fill animation */}
              <clipPath id="fillClip">
                <motion.rect
                  x="0"
                  y="0"
                  width="1200"
                  height="300"
                  style={{ x: fillX }}
                />
              </clipPath>
            </defs>

            {/* Outline stroke */}
            <text
              x="50%"
              y="50%"
              dominantBaseline="middle"
              textAnchor="middle"
              fontSize="200"
              fontWeight="700"
              fontFamily="'Space Grotesk', system-ui, sans-serif"
              fill="none"
              stroke="#e5e7eb"
              strokeWidth="1"
              letterSpacing="-0.02em"
            >
              ASTRA
            </text>

            {/* Filled text with clip path animation */}
            <text
              x="50%"
              y="50%"
              dominantBaseline="middle"
              textAnchor="middle"
              fontSize="200"
              fontWeight="700"
              fontFamily="'Space Grotesk', system-ui, sans-serif"
              fill="#1f2937"
              letterSpacing="-0.02em"
              clipPath="url(#fillClip)"
            >
              ASTRA
            </text>
          </svg>
        </div>

        {/* Subtitle */}
        <motion.p
          style={{ opacity: useTransform(scrollYProgress, [0.3, 0.5], [0, 1]) }}
          className="text-center text-gray-500 text-sm sm:text-base mt-8 font-mono"
        >
          The codebase operating system that never forgets
        </motion.p>
      </div>
    </section>
  );
}
