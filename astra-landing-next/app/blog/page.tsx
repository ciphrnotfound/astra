import { Metadata } from 'next';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';

export const metadata: Metadata = {
  title: 'Blog - Astra',
  description: 'Latest news, updates, and insights from the Astra team.',
};

export default function BlogPage() {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-6xl mx-auto">
          <h1 className="text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Blog
          </h1>
          <p className="text-gray-600 mb-16">Latest news, updates, and insights from the Astra team.</p>

          <div className="bg-white border border-gray-200 p-16 text-center">
            <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
            </svg>
            <h3 className="text-lg font-medium text-gray-900 mb-2">Coming Soon</h3>
            <p className="text-sm text-gray-600">
              We're working on bringing you insightful content about code migrations, AI-powered development, and more.
            </p>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
