import { Metadata } from 'next';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';

export const metadata: Metadata = {
  title: 'Contact - Astra',
  description: 'Get in touch with the Astra team.',
};

export default function ContactPage() {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-4xl mx-auto">
          <div className="text-center mb-16">
            <h1 className="text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Get in Touch
            </h1>
            <p className="text-xl text-gray-600">
              Have questions? We'd love to hear from you.
            </p>
          </div>

          <div className="grid md:grid-cols-2 gap-8 mb-12">
            <div className="bg-white border border-gray-200 p-8">
              <h3 className="text-xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Sales
              </h3>
              <p className="text-gray-600 mb-4">
                Interested in Astra for your team? Let's talk about how we can help.
              </p>
              <a href="mailto:sales@astra.dev" className="text-gray-900 hover:underline font-medium">
                sales@astra.dev
              </a>
            </div>

            <div className="bg-white border border-gray-200 p-8">
              <h3 className="text-xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Support
              </h3>
              <p className="text-gray-600 mb-4">
                Need help with Astra? Our support team is here to assist you.
              </p>
              <a href="mailto:support@astra.dev" className="text-gray-900 hover:underline font-medium">
                support@astra.dev
              </a>
            </div>

            <div className="bg-white border border-gray-200 p-8">
              <h3 className="text-xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                General
              </h3>
              <p className="text-gray-600 mb-4">
                For general inquiries and other questions.
              </p>
              <a href="mailto:hello@astra.dev" className="text-gray-900 hover:underline font-medium">
                hello@astra.dev
              </a>
            </div>

            <div className="bg-white border border-gray-200 p-8">
              <h3 className="text-xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Press
              </h3>
              <p className="text-gray-600 mb-4">
                Media inquiries and press kit requests.
              </p>
              <a href="mailto:press@astra.dev" className="text-gray-900 hover:underline font-medium">
                press@astra.dev
              </a>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
