'use client';

import { motion, useMotionValue, useMotionTemplate } from 'framer-motion';

const Pricing = () => {
  const plans = [
    {
      name: 'Developer',
      price: '$0',
      description: 'For individuals and hobbyists.',
      features: ['Local-first memory', '3 active agents', '1GB persistent storage', 'CLI access'],
      button: 'Start building',
      featured: false
    },
    {
      name: 'Professional',
      price: '$19',
      unit: '/ user / month',
      description: 'For growing teams and startups.',
      features: ['Shared team context', 'Infinite agents', '100GB persistent storage', 'Priority support', 'Webhooks & SDK'],
      button: 'Start free trial',
      featured: true
    },
    {
      name: 'Enterprise',
      price: 'Custom',
      description: 'For large scale organizations.',
      features: ['Custom SLA', 'Dedicated infrastructure', 'Unlimited storage', 'SSO & Advanced Security', 'White-glove onboarding'],
      button: 'Contact sales',
      featured: false
    }
  ];

  return (
    <section className="py-32 px-6 bg-[#faf9f6]">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-20">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            whileInView={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5 }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-gray-200 bg-white text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-8"
          >
            Pricing
          </motion.div>
          
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="text-4xl md:text-6xl font-medium text-gray-900 mb-6 tracking-tight"
            style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}
          >
            Transparent pricing.
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.1 }}
            className="text-gray-600 text-lg"
          >
            Choose the plan that fits your growth.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-3 gap-6">
          {plans.map((plan, i) => (
            <PricingCard key={i} plan={plan} index={i} />
          ))}
        </div>
      </div>
    </section>
  );
};

function PricingCard({ plan, index }: { plan: any, index: number }) {
  const mouseX = useMotionValue(0);
  const mouseY = useMotionValue(0);

  function handleMouseMove({ currentTarget, clientX, clientY }: React.MouseEvent) {
    const { left, top } = currentTarget.getBoundingClientRect();
    mouseX.set(clientX - left);
    mouseY.set(clientY - top);
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, delay: index * 0.1, ease: [0.16, 1, 0.3, 1] }}
      onMouseMove={handleMouseMove}
      className={`group relative ${plan.featured ? 'md:-mt-4' : ''}`}
    >
      <div className={`relative p-10 rounded-2xl border transition-all duration-500 hover:shadow-xl hover:-translate-y-1 overflow-hidden flex flex-col h-full ${
        plan.featured 
          ? 'bg-white border-gray-900 shadow-lg' 
          : 'bg-white border-gray-200'
      }`}>
        {/* Spotlight Effect */}
        <motion.div
          className="pointer-events-none absolute -inset-px rounded-2xl opacity-0 group-hover:opacity-100 transition-opacity duration-500"
          style={{
            background: useMotionTemplate`
              radial-gradient(
                400px circle at ${mouseX}px ${mouseY}px,
                rgba(59, 130, 246, 0.08),
                transparent 80%
              )
            `,
          }}
        />
        
        <div className="relative flex-1 flex flex-col">
          {plan.featured && (
            <div className="absolute -top-10 left-1/2 -translate-x-1/2 bg-gray-900 text-white text-[10px] font-bold uppercase tracking-widest px-4 py-1.5 rounded-full">
              Most Popular
            </div>
          )}
          
          <div className="text-gray-500 font-bold uppercase tracking-widest text-[10px] mb-6">{plan.name}</div>
          
          <div className="flex items-baseline gap-1 mb-4">
            <span className="text-5xl font-medium text-gray-900 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              {plan.price}
            </span>
            {plan.unit && <span className="text-gray-500 text-sm">{plan.unit}</span>}
          </div>
          
          <p className="text-gray-600 mb-8">{plan.description}</p>
          
          <ul className="space-y-3 mb-10 flex-1">
            {plan.features.map((feature: string, j: number) => (
              <li key={j} className="flex items-center gap-3 text-sm text-gray-700">
                <div className={`w-1.5 h-1.5 rounded-full ${plan.featured ? 'bg-gray-900' : 'bg-gray-400'}`} />
                {feature}
              </li>
            ))}
          </ul>

          <button className={`relative group overflow-hidden w-full py-3 rounded text-sm font-medium transition-all ${
            plan.featured 
              ? 'bg-gray-900 text-white hover:shadow-lg' 
              : 'border border-gray-900 text-gray-900 hover:text-white'
          }`}>
            <span className="relative z-10">{plan.button}</span>
            <div className={`absolute inset-0 transform transition-transform duration-300 ease-out ${
              plan.featured
                ? 'bg-gray-800 scale-x-0 group-hover:scale-x-100 origin-left'
                : 'bg-gray-900 translate-y-full group-hover:translate-y-0'
            }`} />
          </button>
        </div>
      </div>
    </motion.div>
  );
}

export default Pricing;
