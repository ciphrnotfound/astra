import { Metadata } from 'next';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import AboutHero from '@/components/about/AboutHero';
import Mission from '@/components/about/Mission';
import Team from '@/components/about/Team';
import Values from '@/components/about/Values';

export const metadata: Metadata = {
  title: 'About - Astra',
  description: 'Learn about Astra and our mission to transform code migration.',
};

export default function AboutPage() {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      <main>
        <AboutHero />
        <Mission />
        <Values />
        <Team />
      </main>
      <Footer />
    </div>
  );
}
