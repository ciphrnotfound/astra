'use client';

import { motion } from 'framer-motion';
import { Check, X, Minus } from 'lucide-react';

const ComparisonTable = () => {
  const rows = [
    { feature: 'Retrieval Speed', standard: '2.5s - 4.0s', astra: 'Sub-second (<0.8s)', status: 'win' },
    { feature: 'Contextual Awareness', standard: 'Limited window', astra: 'Infinite long-term', status: 'win' },
    { feature: 'Search Accuracy', standard: '82%', astra: '99.9%', status: 'win' },
    { feature: 'Deployment', standard: 'Complex infra', astra: 'Single CLI command', status: 'win' },
    { feature: 'Data Privacy', standard: 'Cloud-dependent', astra: 'Local-first option', status: 'win' },
  ];

  return (
    <section className="py-32 bg-gray-50/50 font-sans">
      <div className="max-w-5xl mx-auto px-6">
        <div className="text-center mb-16">
          <h2 className="text-4xl font-bold text-black mb-4 tracking-[-0.03em]" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
            Compare the difference.
          </h2>
          <p className="text-gray-500 font-medium">Why the world's best teams choose Astra.</p>
        </div>

        <div className="bg-white rounded-[2.5rem] border border-gray-100 overflow-hidden shadow-xl">
           <table className="w-full text-left">
              <thead>
                 <tr className="border-b border-gray-100 bg-gray-50/50">
                    <th className="px-10 py-8 text-sm font-bold text-gray-400 uppercase tracking-widest">Feature</th>
                    <th className="px-10 py-8 text-sm font-bold text-gray-400 uppercase tracking-widest text-center">Standard RAG</th>
                    <th className="px-10 py-8 text-sm font-bold text-[#2F55FF] uppercase tracking-widest text-center bg-blue-50/30">Astra</th>
                 </tr>
              </thead>
              <tbody>
                 {rows.map((row, i) => (
                    <tr key={i} className="border-b border-gray-100 last:border-0 hover:bg-gray-50/50 transition-colors">
                       <td className="px-10 py-6 text-black font-bold" style={{ fontFamily: 'var(--font-space-grotesk)' }}>{row.feature}</td>
                       <td className="px-10 py-6 text-center text-gray-400 font-medium">{row.standard}</td>
                       <td className="px-10 py-6 text-center text-[#2F55FF] font-black bg-blue-50/10">
                          <div className="flex items-center justify-center gap-2">
                             <Check className="w-5 h-5" />
                             {row.astra}
                          </div>
                       </td>
                    </tr>
                 ))}
              </tbody>
           </table>
           
           <div className="p-8 bg-blue-50/30 border-t border-blue-100/50 text-center">
              <p className="text-sm font-bold text-[#2F55FF] uppercase tracking-widest">Verdict: Astra is 5x faster and 20% more accurate.</p>
           </div>
        </div>
      </div>
    </section>
  );
};

export default ComparisonTable;
