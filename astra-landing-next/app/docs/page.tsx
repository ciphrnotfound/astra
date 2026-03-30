import { Metadata } from 'next';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'Documentation - Astra',
  description: 'Get started with Astra - installation, quick start, and command reference.',
};

export default function DocsPage() {
  return (
    <section className="border-b border-gray-200 bg-white">
      <div className="max-w-4xl mx-auto px-8 py-16">
        <h1 className="text-4xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Documentation
        </h1>
        <p className="text-lg text-gray-600 mb-8">
          Astra is a cross-language code migration CLI that helps you refactor and migrate code between different programming languages with semantic understanding.
        </p>
        <div className="bg-[#faf9f6] border border-gray-200 p-6 mb-12">
          <p className="text-sm text-gray-600">
            <strong className="text-gray-900">Note:</strong> Astra is currently in active development. Some features may be experimental.
          </p>
        </div>

        <div className="space-y-8">
          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              What is Astra?
            </h2>
            <p className="text-gray-600 mb-4">
              Astra is a powerful CLI tool designed to help developers migrate code between different programming languages while preserving semantic meaning and structure. Unlike simple syntax translators, Astra understands the intent behind your code and generates idiomatic code in the target language.
            </p>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Key Features
            </h2>
            <div className="grid gap-4">
              <div className="border border-gray-200 p-6">
                <h3 className="font-medium text-gray-900 mb-2">Semantic Understanding</h3>
                <p className="text-sm text-gray-600">
                  Astra analyzes your code at the semantic level, understanding types, scopes, and relationships to ensure accurate migrations.
                </p>
              </div>
              <div className="border border-gray-200 p-6">
                <h3 className="font-medium text-gray-900 mb-2">Type-Safe Migrations</h3>
                <p className="text-sm text-gray-600">
                  Preserve type safety across languages with intelligent type mapping and inference.
                </p>
              </div>
              <div className="border border-gray-200 p-6">
                <h3 className="font-medium text-gray-900 mb-2">Idiomatic Output</h3>
                <p className="text-sm text-gray-600">
                  Generated code follows best practices and idioms of the target language.
                </p>
              </div>
            </div>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Getting Started
            </h2>
            <p className="text-gray-600 mb-4">
              Ready to start migrating your code? Check out our installation guide and quick start tutorial:
            </p>
            <div className="flex gap-4">
              <Link
                href="/docs/installation"
                className="relative group overflow-hidden bg-gray-900 text-white px-6 py-3 text-sm font-medium transition-all hover:shadow-lg"
              >
                <span className="relative z-10">Installation</span>
                <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
              </Link>
              <Link
                href="/docs/quick-start"
                className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 text-sm font-medium transition-all hover:text-white"
              >
                <span className="relative z-10">Quick Start</span>
                <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
              </Link>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
