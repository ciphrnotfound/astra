import Link from 'next/link';

export default function NotFound() {
  return (
    <div className="min-h-screen bg-[#faf9f6] flex items-center justify-center px-6">
      <div className="max-w-2xl w-full text-center">
        {/* Logo */}
        <Link href="/" className="inline-flex items-center gap-2 mb-12">
          <div className="w-42 h-42  flex items-center justify-center shrink-0">
            <img src="/astra-logo-2.png" alt="Astra Logo" className="w-full h-full object-contain" />
          </div>
          
        </Link>

        {/* 404 Content */}
        <div className="bg-white border border-gray-200 p-12">
          <h1 className="text-8xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            404
          </h1>
          <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Page Not Found
          </h2>
          <p className="text-gray-600 mb-8 max-w-md mx-auto">
            The page you're looking for doesn't exist or has been moved.
          </p>

          {/* Actions */}
          <div className="flex gap-4 justify-center">
            <Link
              href="/"
              className="relative group overflow-hidden bg-gray-900 text-white px-6 py-3 text-sm font-medium transition-all hover:shadow-lg"
            >
              <span className="relative z-10">Go Home</span>
              <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
            </Link>
            <Link
              href="/docs"
              className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 text-sm font-medium transition-all hover:text-white"
            >
              <span className="relative z-10">Documentation</span>
              <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
            </Link>
          </div>
        </div>

        {/* Quick Links */}
        <div className="mt-8 text-sm text-gray-600">
          <p className="mb-4">Looking for something specific?</p>
          <div className="flex gap-6 justify-center">
            <Link href="/about" className="hover:text-gray-900 transition-colors">
              About
            </Link>
            <Link href="/research" className="hover:text-gray-900 transition-colors">
              Research
            </Link>
            <Link href="/contact" className="hover:text-gray-900 transition-colors">
              Contact
            </Link>
            <Link href="/signin" className="hover:text-gray-900 transition-colors">
              Sign In
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
