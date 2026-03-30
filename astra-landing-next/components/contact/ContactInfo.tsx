'use client';

import { motion } from 'framer-motion';
import { Mail, MessageCircle, MapPin } from 'lucide-react';

const contactMethods = [
  {
    icon: Mail,
    title: 'Email',
    detail: 'hello@astra.dev',
    description: 'Send us an email anytime',
  },
  {
    icon: MessageCircle,
    title: 'Live chat',
    detail: 'Start a conversation',
    description: 'Available Mon-Fri, 9am-6pm PST',
  },
  {
    icon: MapPin,
    title: 'Office',
    detail: 'San Francisco, CA',
    description: 'Visit us in person',
  },
];

export default function ContactInfo() {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, delay: 0.2 }}
    >
      <h2 className="text-3xl font-medium text-gray-900 mb-8 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
        Other ways to reach us
      </h2>

      <div className="space-y-6">
        {contactMethods.map((method, index) => (
          <div
            key={method.title}
            className="p-6 rounded-2xl bg-white border border-gray-200"
          >
            <div className="flex items-start gap-4">
              <div className="w-12 h-12 rounded-full bg-gray-100 flex items-center justify-center shrink-0">
                <method.icon className="w-5 h-5 text-gray-900" />
              </div>
              <div>
                <h3 className="text-lg font-medium text-gray-900 mb-1">
                  {method.title}
                </h3>
                <p className="text-gray-900 mb-1">
                  {method.detail}
                </p>
                <p className="text-sm text-gray-600">
                  {method.description}
                </p>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-12 p-6 rounded-2xl bg-gray-100 border border-gray-200">
        <h3 className="text-lg font-medium text-gray-900 mb-2">
          Enterprise inquiries
        </h3>
        <p className="text-gray-600 mb-4">
          Looking for custom solutions or enterprise support? Our team is here to help.
        </p>
        <a
          href="mailto:enterprise@astra.dev"
          className="text-sm text-gray-900 hover:underline"
        >
          enterprise@astra.dev
        </a>
      </div>
    </motion.div>
  );
}
