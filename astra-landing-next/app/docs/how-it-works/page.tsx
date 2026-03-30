import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'How It Works - Astra Documentation',
  description: 'Learn how Astra migrates code between languages.',
};

export default function HowItWorksPage() {
  return (
    <section className="bg-[#faf9f6]">
      <div className="max-w-4xl mx-auto px-8 py-16">
        <h1 className="text-4xl font-medium text-gray-900 mb-4 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          How It Works
        </h1>
        <p className="text-lg text-gray-600 mb-12">
          Astra uses a multi-stage pipeline to ensure accurate code migration.
        </p>

        <div className="space-y-6">
          <div className="bg-white border border-gray-200 p-6">
            <h4 className="font-medium text-gray-900 mb-2">1. Parsing</h4>
            <p className="text-sm text-gray-600">
              Astra parses your source code into an Abstract Syntax Tree (AST) using language-specific parsers.
            </p>
          </div>

          <div className="bg-white border border-gray-200 p-6">
            <h4 className="font-medium text-gray-900 mb-2">2. Semantic Analysis</h4>
            <p className="text-sm text-gray-600">
              The AST is analyzed to understand types, scopes, and relationships between code elements.
            </p>
          </div>

          <div className="bg-white border border-gray-200 p-6">
            <h4 className="font-medium text-gray-900 mb-2">3. Transformation</h4>
            <p className="text-sm text-gray-600">
              Code patterns are transformed to equivalent patterns in the target language.
            </p>
          </div>

          <div className="bg-white border border-gray-200 p-6">
            <h4 className="font-medium text-gray-900 mb-2">4. Code Generation</h4>
            <p className="text-sm text-gray-600">
              The transformed AST is converted back to source code in the target language.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
