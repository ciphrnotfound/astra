'use client';

import { motion } from 'framer-motion';

const companies = [
  { name: 'Vercel', logo: 'V' },
  { name: 'Linear', logo: 'L' },
  { name: 'Stripe', logo: 'S' },
  { name: 'GitHub', logo: 'G' },
  { name: 'Notion', logo: 'N' },
  { name: 'Figma', logo: 'F' },
];

const testimonials = [
  {
    quote: "Astra cut our migration time from 6 months to 3 weeks. The type safety guarantees gave us confidence to ship faster.",
    author: "Sarah Chen",
    role: "Engineering Lead",
    company: "TechCorp",
    avatar: "SC",
  },
  {
    quote: "Time travel debugging changed how we approach complex bugs. We can now see exactly what happened, not just guess.",
    author: "Marcus Rodriguez",
    role: "Senior Developer",
    company: "DevTools Inc",
    avatar: "MR",
  },
  {
    quote: "The semantic understanding is incredible. Astra caught edge cases our team missed during manual refactoring.",
    author: "Emily Watson",
    role: "CTO",
    company: "StartupXYZ",
    avatar: "EW",
  },
];

export default function SocialProof() {
  return (
    <section className="py-32 px-6 bg-white">
      <div className="max-w-6xl mx-auto">
        {/* Companies */}
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8 }}
          className="text-center mb-20"
        >
          <p className="text-sm text-gray-500 mb-8">Trusted by teams at</p>
          <div className="flex flex-wrap items-center justify-center gap-12">
            {companies.map((company, index) => (
              <motion.div
                key={company.name}
                initial={{ opacity: 0 }}
                whileInView={{ opacity: 1 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
                className="flex items-center justify-center"
              >
                <div className="w-12 h-12 bg-[#faf9f6] border border-gray-200 flex items-center justify-center transition-colors hover:border-gray-900">
                  <span className="text-lg font-medium text-gray-700" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                    {company.logo}
                  </span>
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>

        {/* Testimonials */}
        <div className="grid md:grid-cols-3 gap-6">
          {testimonials.map((testimonial, index) => (
            <motion.div
              key={testimonial.author}
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: index * 0.15 }}
            >
              <div className="bg-[#faf9f6] border border-gray-200 p-8 h-full transition-all duration-300 hover:border-gray-900">
                <div className="mb-6">
                  <svg className="w-8 h-8 text-gray-300" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M14.017 21v-7.391c0-5.704 3.731-9.57 8.983-10.609l.995 2.151c-2.432.917-3.995 3.638-3.995 5.849h4v10h-9.983zm-14.017 0v-7.391c0-5.704 3.748-9.57 9-10.609l.996 2.151c-2.433.917-3.996 3.638-3.996 5.849h3.983v10h-9.983z" />
                  </svg>
                </div>
                
                <p className="text-gray-700 leading-relaxed mb-6">
                  {testimonial.quote}
                </p>
                
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-gray-900 flex items-center justify-center">
                    <span className="text-xs font-medium text-white">
                      {testimonial.avatar}
                    </span>
                  </div>
                  <div>
                    <div className="text-sm font-medium text-gray-900">
                      {testimonial.author}
                    </div>
                    <div className="text-xs text-gray-500">
                      {testimonial.role} at {testimonial.company}
                    </div>
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
