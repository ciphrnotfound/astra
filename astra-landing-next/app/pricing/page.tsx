import { Metadata } from 'next';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import { Check } from 'lucide-react';

export const metadata: Metadata = {
  title: 'Pricing - Astra',
  description: 'Simple, transparent pricing for teams of all sizes.',
};

const plans = [
  {
    name: 'Free',
    price: '$0',
    description: 'Perfect for trying out Astra',
    features: [
      '10 migrations per month',
      'Basic language support',
      'Community support',
      'CLI access',
    ],
  },
  {
    name: 'Pro',
    price: '$29',
    description: 'For professional developers',
    features: [
      'Unlimited migrations',
      'All language support',
      'Priority support',
      'Advanced refactoring',
      'API access',
      'Team collaboration',
    ],
    popular: true,
  },
  {
    name: 'Enterprise',
    price: 'Custom',
    description: 'For large teams and organizations',
    features: [
      'Everything in Pro',
      'Dedicated support',
      'Custom integrations',
      'SLA guarantee',
      'On-premise deployment',
      'Advanced security',
    ],
  },
];

export default function PricingPage() {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16">
            <h1 className="text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Simple, Transparent Pricing
            </h1>
            <p className="text-xl text-gray-600 max-w-2xl mx-auto">
              Choose the plan that's right for you. All plans include a 14-day free trial.
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {plans.map((plan) => (
              <div
                key={plan.name}
                className={`relative bg-white border ${
                  plan.popular ? 'border-gray-900' : 'border-gray-200'
                } p-8`}
              >
                {plan.popular && (
                  <div className="absolute -top-4 left-1/2 -translate-x-1/2 px-3 py-1 bg-gray-900 text-white text-xs font-medium">
                    MOST POPULAR
                  </div>
                )}
                
                <h3 className="text-2xl font-medium text-gray-900 mb-2" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {plan.name}
                </h3>
                <p className="text-gray-600 text-sm mb-6">{plan.description}</p>
                
                <div className="mb-6">
                  <span className="text-4xl font-medium text-gray-900" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                    {plan.price}
                  </span>
                  {plan.price !== 'Custom' && (
                    <span className="text-gray-600">/month</span>
                  )}
                </div>

                <button
                  className={`relative group overflow-hidden w-full px-6 py-3 text-sm font-medium transition-all mb-8 ${
                    plan.popular
                      ? 'bg-gray-900 text-white hover:shadow-lg'
                      : 'border border-gray-900 text-gray-900 hover:text-white'
                  }`}
                >
                  <span className="relative z-10">
                    {plan.price === 'Custom' ? 'Contact Sales' : 'Get Started'}
                  </span>
                  {!plan.popular && (
                    <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
                  )}
                </button>

                <ul className="space-y-3">
                  {plan.features.map((feature) => (
                    <li key={feature} className="flex items-start gap-3">
                      <Check className="w-5 h-5 text-gray-900 shrink-0 mt-0.5" />
                      <span className="text-sm text-gray-600">{feature}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
