import { Metadata } from 'next';
import Link from 'next/link';
import SignInForm from '@/components/auth/SignInForm';

export const metadata: Metadata = {
  title: 'Sign In - Astra',
  description: 'Sign in to your Astra account',
};

// Disable static generation for this page
export const dynamic = 'force-dynamic';

export default function SignInPage() {
  return (
    <div className="min-h-screen bg-[#faf9f6] flex items-center justify-center px-6 py-12">
      <div className="w-full max-w-md">
        {/* Logo */}
        <Link href="/" className="flex items-center justify-center gap-2 mb-12">
          <div className="w-10 h-10 md:h-32 md:w-32  flex items-center justify-center shrink-0">
            <img src="/astra-logo-2.png" alt="Astra Logo" className="w-full h-full object-contain" />
          </div>
          {/* <span className="text-xl font-medium text-gray-900 tracking-tight" style={{ fontFamily: 'var(--font-space-grotesk)' }}>
            Astra
          </span> */}
        </Link>

        {/* Sign In Form */}
        <div className="bg-white border border-gray-200 p-8">
          <h1 className="text-3xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Welcome back
          </h1>
          <p className="text-gray-600 mb-8">
            Sign in to your account to continue
          </p>

          <SignInForm />

          <div className="mt-6 text-center">
            <p className="text-sm text-gray-600">
              Don't have an account?{' '}
              <Link href="/signup" className="text-gray-900 font-medium hover:underline">
                Sign up
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
