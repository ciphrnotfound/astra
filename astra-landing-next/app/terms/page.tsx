import { Metadata } from 'next';
import Navbar from '@/components/Navbar';
import Footer from '@/components/Footer';

export const metadata: Metadata = {
  title: 'Terms of Service - Astra',
  description: 'Terms and conditions for using Astra.',
};

export default function TermsPage() {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <Navbar />
      
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-5xl font-medium text-gray-900 mb-6 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Terms of Service
          </h1>
          <p className="text-gray-600 mb-12">Last updated: March 24, 2026</p>

          <div className="prose prose-gray max-w-none">
            <section className="mb-12">
              <h2 className="text-2xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Agreement to Terms
              </h2>
              <p className="text-gray-600 leading-relaxed">
                By accessing or using Astra, you agree to be bound by these Terms of Service and all applicable laws and regulations. If you do not agree with any of these terms, you are prohibited from using or accessing this service.
              </p>
            </section>

            <section className="mb-12">
              <h2 className="text-2xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Use License
              </h2>
              <p className="text-gray-600 leading-relaxed mb-4">
                Permission is granted to temporarily use Astra for personal or commercial purposes. This is the grant of a license, not a transfer of title, and under this license you may not:
              </p>
              <ul className="list-disc pl-6 text-gray-600 space-y-2">
                <li>Modify or copy the materials</li>
                <li>Use the materials for any commercial purpose without a valid license</li>
                <li>Attempt to reverse engineer any software contained in Astra</li>
                <li>Remove any copyright or proprietary notations</li>
                <li>Transfer the materials to another person or mirror on any other server</li>
              </ul>
            </section>

            <section className="mb-12">
              <h2 className="text-2xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                User Accounts
              </h2>
              <p className="text-gray-600 leading-relaxed">
                You are responsible for maintaining the confidentiality of your account and password. You agree to accept responsibility for all activities that occur under your account.
              </p>
            </section>

            <section className="mb-12">
              <h2 className="text-2xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Disclaimer
              </h2>
              <p className="text-gray-600 leading-relaxed">
                The materials on Astra are provided on an 'as is' basis. Astra makes no warranties, expressed or implied, and hereby disclaims and negates all other warranties including, without limitation, implied warranties or conditions of merchantability, fitness for a particular purpose, or non-infringement of intellectual property or other violation of rights.
              </p>
            </section>

            <section className="mb-12">
              <h2 className="text-2xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Limitations
              </h2>
              <p className="text-gray-600 leading-relaxed">
                In no event shall Astra or its suppliers be liable for any damages (including, without limitation, damages for loss of data or profit, or due to business interruption) arising out of the use or inability to use Astra.
              </p>
            </section>

            <section className="mb-12">
              <h2 className="text-2xl font-medium text-gray-900 mb-4" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                Contact
              </h2>
              <p className="text-gray-600 leading-relaxed">
                Questions about the Terms of Service should be sent to{' '}
                <a href="mailto:legal@astra.dev" className="text-gray-900 hover:underline">
                  legal@astra.dev
                </a>
              </p>
            </section>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
