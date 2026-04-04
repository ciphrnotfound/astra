import Features from '@/components/Features';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';

export default function FeaturesPage() {
  return (
    <main className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      <div className="pt-20">
        <Features />
      </div>
      <Footer />
    </main>
  );
}
