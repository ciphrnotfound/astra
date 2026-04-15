import Navbar from '@/components/Navbar';
import Hero from '@/components/Hero';
import Features from '@/components/Features';
import WhatWeDo from '@/components/WhatWeDo';
import LiveDemo from '@/components/LiveDemo';
import UseCases from '@/components/UseCases';
import TechStack from '@/components/TechStack';
import Metrics from '@/components/Metrics';
import Stats from '@/components/Stats';
import DarkSection from '@/components/DarkSection';
import Pricing from '@/components/Pricing';
import Footer from '@/components/Footer';
import GiantAstra from '@/components/GiantAstra';

export default function Home() {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      <main>
        <Hero />
        <Features />
        <LiveDemo />
        <WhatWeDo />
        <UseCases />
        <TechStack />
        <Metrics />
        <Stats />
        <DarkSection />
        <Pricing />
      </main>
      <Footer />
      <GiantAstra />
    </div>
  );
}
