import { Metadata } from 'next';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';
import ResearchHero from '@/components/research/ResearchHero';
import ResearchAreas from '@/components/research/ResearchAreas';
import Publications from '@/components/research/Publications';

export const metadata: Metadata = {
  title: 'Research - Astra',
  description: 'Explore our research in AI-powered code migration and semantic analysis.',
};

export default function ResearchPage() {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      <main>
        <ResearchHero />
        <ResearchAreas />
        <Publications />
      </main>
      <Footer />
    </div>
  );
}
