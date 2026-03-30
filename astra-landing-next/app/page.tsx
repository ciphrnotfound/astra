import Navbar from '@/components/Navbar';
import Hero from '@/components/Hero';
import Features from '@/components/Features';
import CodeShowcase from '@/components/CodeShowcase';
import WhatWeDo from '@/components/WhatWeDo';
import UseCases from '@/components/UseCases';
import SocialProof from '@/components/SocialProof';
import Stats from '@/components/Stats';
import DarkSection from '@/components/DarkSection';
import Testimonials from '@/components/Testimonials';
import Pricing from '@/components/Pricing';
import Footer from '@/components/Footer';

export default function Home() {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      <main>
        <Hero />
        <Features />
        <CodeShowcase />
        <WhatWeDo />
        <UseCases />
        <SocialProof />
        <Stats />
        <section className="px-6 py-20 bg-white border-y border-gray-200">
          <div className="max-w-6xl mx-auto grid lg:grid-cols-2 gap-10 items-start">
            <div>
              <p className="text-xs uppercase tracking-widest text-gray-500 mb-3">Main Astra Project</p>
              <h3 className="text-3xl md:text-4xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Connected to the real Astra workspace
              </h3>
              <p className="text-gray-600 leading-relaxed mb-5">
                The landing experience now reflects the production project structure: core engine, CLI surface, team workflows, migrations, and model integrations.
              </p>
              <div className="grid sm:grid-cols-2 gap-3 text-sm">
                <div className="border border-gray-200 bg-[#faf9f6] p-3">core/ — agent engine, memory, graph</div>
                <div className="border border-gray-200 bg-[#faf9f6] p-3">cli/ — commands, auth, team sync</div>
                <div className="border border-gray-200 bg-[#faf9f6] p-3">lsp/ — editor protocol integration</div>
                <div className="border border-gray-200 bg-[#faf9f6] p-3">hooks/ — automation and checks</div>
              </div>
            </div>
            <div className="border border-gray-200 bg-[#111] text-[#f6f6f6] p-5 overflow-x-auto">
              <pre className="text-sm leading-7">
{`astra auth login-web
astra team status
astra team sync --cloud
:index
:memory compact
teach me rust async patterns`}
              </pre>
            </div>
          </div>
        </section>
        <DarkSection />
        <Testimonials />
        <Pricing />
      </main>
      <Footer />
    </div>
  );
}
