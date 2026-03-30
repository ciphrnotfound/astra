import { Metadata } from 'next';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'Quick Start - Astra Documentation',
  description: 'Get started with Astra in minutes with this quick start guide.',
};

export default function QuickStartPage() {
  return (
    <section className="bg-white">
      <div className="max-w-4xl mx-auto px-8 py-16">
        <h1 className="text-4xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Quick Start
        </h1>
        <p className="text-lg text-gray-600 mb-12">
          Learn the basics of Astra with these simple examples.
        </p>

        <div className="space-y-12">
          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Your First Migration
            </h2>
            <p className="text-gray-600 mb-4">
              Let's migrate a simple TypeScript file to Rust. Create a file called <code className="bg-gray-900 text-white px-2 py-1 text-sm">utils.ts</code>:
            </p>
            <div className="bg-gray-900 text-white p-4 font-mono text-sm overflow-x-auto mb-4">
              <pre>{`export function add(a: number, b: number): number {
  return a + b;
}

export function multiply(a: number, b: number): number {
  return a * b;
}`}</pre>
            </div>
            <p className="text-gray-600 mb-4">
              Now run the migration command:
            </p>
            <div className="bg-gray-900 text-white p-4 font-mono text-sm overflow-x-auto">
              <code>astra migrate --from typescript --to rust --input utils.ts --output utils.rs</code>
            </div>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Analyzing Your Code
            </h2>
            <p className="text-gray-600 mb-4">
              Before migrating a large codebase, it's helpful to analyze it first:
            </p>
            <div className="bg-gray-900 text-white p-4 font-mono text-sm overflow-x-auto">
              <code>astra analyze --input src/ --language typescript</code>
            </div>
            <p className="text-sm text-gray-600 mt-4">
              This will show you statistics about your codebase, including complexity metrics and potential migration challenges.
            </p>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Validating Output
            </h2>
            <p className="text-gray-600 mb-4">
              After migration, validate that the output code is correct:
            </p>
            <div className="bg-gray-900 text-white p-4 font-mono text-sm overflow-x-auto">
              <code>astra validate --input utils.rs --language rust</code>
            </div>
            <p className="text-sm text-gray-600 mt-4">
              This checks for syntax errors, type issues, and other potential problems.
            </p>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Batch Migration
            </h2>
            <p className="text-gray-600 mb-4">
              Migrate an entire directory of files:
            </p>
            <div className="bg-gray-900 text-white p-4 font-mono text-sm overflow-x-auto">
              <code>astra migrate --from typescript --to rust --input src/ --output target/</code>
            </div>
            <div className="bg-[#faf9f6] border border-gray-200 p-4 mt-4">
              <p className="text-sm text-gray-600">
                <strong className="text-gray-900">Tip:</strong> Astra preserves directory structure when migrating multiple files.
              </p>
            </div>
          </div>

          <div className="border-t border-gray-200 pt-8">
            <h2 className="text-xl font-medium text-gray-900 mb-4">Next Steps</h2>
            <p className="text-gray-600 mb-4">
              Now that you know the basics, explore more advanced features:
            </p>
            <div className="flex gap-4">
              <Link
                href="/docs/configuration"
                className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 text-sm font-medium transition-all hover:text-white"
              >
                <span className="relative z-10">Configuration</span>
                <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
              </Link>
              <Link
                href="/docs/commands/migrate"
                className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 text-sm font-medium transition-all hover:text-white"
              >
                <span className="relative z-10">Command Reference</span>
                <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
              </Link>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
