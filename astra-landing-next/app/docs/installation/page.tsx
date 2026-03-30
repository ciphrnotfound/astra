import { Metadata } from 'next';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'Installation - Astra Documentation',
  description: 'Learn how to install Astra CLI on your system.',
};

export default function InstallationPage() {
  return (
    <section className="bg-[#faf9f6]">
      <div className="max-w-4xl mx-auto px-8 py-16">
        <h1 className="text-4xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Installation
        </h1>
        <p className="text-lg text-gray-600 mb-12">
          Get Astra up and running on your system in minutes.
        </p>

        <div className="space-y-12">
          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Using Cargo
            </h2>
            <p className="text-gray-600 mb-4">
              The easiest way to install Astra is through Cargo, Rust's package manager:
            </p>
            <div className="bg-gray-900 text-white p-4 font-mono text-sm overflow-x-auto">
              <code>cargo install astra-cli</code>
            </div>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              From Source
            </h2>
            <p className="text-gray-600 mb-4">
              To build Astra from source, clone the repository and build with Cargo:
            </p>
            <div className="bg-gray-900 text-white p-4 font-mono text-sm overflow-x-auto space-y-2">
              <div><code>git clone https://github.com/yourusername/astra.git</code></div>
              <div><code>cd astra</code></div>
              <div><code>cargo build --release</code></div>
            </div>
            <p className="text-sm text-gray-600 mt-4">
              The compiled binary will be available at <code className="bg-gray-900 text-white px-2 py-1 text-xs">target/release/astra</code>
            </p>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              Verify Installation
            </h2>
            <p className="text-gray-600 mb-4">
              After installation, verify that Astra is correctly installed:
            </p>
            <div className="bg-gray-900 text-white p-4 font-mono text-sm overflow-x-auto">
              <code>astra --version</code>
            </div>
            <p className="text-sm text-gray-600 mt-4">
              You should see the version number printed to the console.
            </p>
          </div>

          <div>
            <h2 className="text-2xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
              System Requirements
            </h2>
            <div className="bg-white border border-gray-200 p-6">
              <ul className="space-y-2 text-sm text-gray-600">
                <li>• Rust 1.70 or higher</li>
                <li>• 4GB RAM minimum (8GB recommended)</li>
                <li>• 500MB disk space</li>
                <li>• macOS, Linux, or Windows</li>
              </ul>
            </div>
          </div>

          <div className="border-t border-gray-200 pt-8">
            <h2 className="text-xl font-medium text-gray-900 mb-4">Next Steps</h2>
            <p className="text-gray-600 mb-4">
              Now that you have Astra installed, check out the Quick Start guide to learn how to use it:
            </p>
            <Link
              href="/docs/quick-start"
              className="relative group overflow-hidden bg-gray-900 text-white px-6 py-3 text-sm font-medium transition-all hover:shadow-lg inline-block"
            >
              <span className="relative z-10">Quick Start Guide →</span>
              <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}
