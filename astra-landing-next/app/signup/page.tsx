import { Metadata } from 'next';
import Link from 'next/link';
import SignUpForm from '@/components/auth/SignUpForm';

export const metadata: Metadata = {
  title: 'Sign Up - Astra',
  description: 'Create your Astra account',
};

// Disable static generation for this page
export const dynamic = 'force-dynamic';

export default function SignUpPage() {
  return (
    <div className="min-h-screen bg-[#faf9f6] flex items-center justify-center px-6 py-12">
      <div className="w-full max-w-md">
        {/* Logo */}
        <Link href="/" className="flex items-center justify-center mb-12">
          <div className="w-10 h-10 border border-gray-900 bg-white flex items-center justify-center shrink-0">
            <img src="/astra-logo-2.jpg" alt="Astra Logo" className="w-full h-full object-contain" />
          </div>
          <span className="text-xl font-bold text-gray-900 tracking-tight" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
            Astra
          </span>
        </Link>

        {/* Sign Up Form */}
        <div className="bg-white border border-gray-200 p-8">
          <h1 className="text-3xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Get started
          </h1>
          <p className="text-gray-600 mb-8">
            Create your account to start using Astra
          </p>

          <SignUpForm />

          <div className="mt-6 text-center">
            <p className="text-sm text-gray-600">
              Already have an account?{' '}
              <Link href="/signin" className="text-gray-900 font-medium hover:underline">
                Sign in
              </Link>
            </p>
          </div>
        </div>

        {/* Back to home */}
        <div className="mt-6 text-center">
          <Link href="/" className="text-sm text-gray-600 hover:text-gray-900 transition-colors">
            ← Back to home
          </Link>
        </div>
      </div>
    </div>
  );
}
