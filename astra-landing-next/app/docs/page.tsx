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
              Installation
            </h2>
            <p className="text-gray-600 mb-4">
              Install Astra globally using npm:
            </p>
            <div className="bg-gray-900 text-white p-6 mb-6 font-mono text-sm">
              <code>npm install -g @astra/cli</code>
            </div>
            <p className="text-gray-600 mb-4">
              Or use npx to run without installing:
            </p>
            <div className="bg-gray-900 text-white p-6 mb-6 font-mono text-sm">
              <code>npx @astra/cli migrate --from typescript --to rust</code>
            </div>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Quick Start
            </h2>
            <p className="text-gray-600 mb-4">
              Migrate your first file in seconds:
            </p>
            <div className="space-y-4">
              <div>
                <p className="text-sm font-medium text-gray-700 mb-2">1. Initialize Astra in your project</p>
                <div className="bg-gray-900 text-white p-4 font-mono text-sm">
                  <code>astra init</code>
                </div>
              </div>
              <div>
                <p className="text-sm font-medium text-gray-700 mb-2">2. Run a migration</p>
                <div className="bg-gray-900 text-white p-4 font-mono text-sm">
                  <code>astra migrate --from typescript --to rust src/</code>
                </div>
              </div>
              <div>
                <p className="text-sm font-medium text-gray-700 mb-2">3. Review and apply changes</p>
                <div className="bg-gray-900 text-white p-4 font-mono text-sm">
                  <code>astra review</code>
                </div>
              </div>
            </div>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Common Commands
            </h2>
            <div className="space-y-3">
              <div className="border border-gray-200 p-4">
                <code className="text-sm font-mono text-gray-900">astra migrate</code>
                <p className="text-sm text-gray-600 mt-2">Migrate code between languages</p>
              </div>
              <div className="border border-gray-200 p-4">
                <code className="text-sm font-mono text-gray-900">astra analyze</code>
                <p className="text-sm text-gray-600 mt-2">Analyze codebase health and dependencies</p>
              </div>
              <div className="border border-gray-200 p-4">
                <code className="text-sm font-mono text-gray-900">astra sync</code>
                <p className="text-sm text-gray-600 mt-2">Sync data to dashboard</p>
              </div>
              <div className="border border-gray-200 p-4">
                <code className="text-sm font-mono text-gray-900">astra health</code>
                <p className="text-sm text-gray-600 mt-2">Check codebase health metrics</p>
              </div>
            </div>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Need Help?
            </h2>
            <p className="text-gray-600 mb-4">
              Explore our comprehensive documentation or join our community:
            </p>
            <div className="flex gap-4">
              <Link
                href="https://github.com/astra-cli/astra"
                className="relative group overflow-hidden bg-gray-900 text-white px-6 py-3 text-sm font-medium transition-all hover:shadow-lg"
              >
                <span className="relative z-10">GitHub</span>
                <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
              </Link>
              <Link
                href="/contact"
                className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 text-sm font-medium transition-all hover:text-white"
              >
                <span className="relative z-10">Contact Support</span>
                <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
              </Link>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
