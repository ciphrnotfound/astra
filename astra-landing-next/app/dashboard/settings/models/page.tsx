import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Models - Settings - Astra Dashboard',
  description: 'Bring your own API keys',
};

export default function ModelsPage() {
  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          AI Models
        </h1>
        <p className="text-gray-600">BYOK — add your own API keys (Groq, OpenAI, Gemini, Ollama)</p>
      </div>

      <div className="bg-white border border-gray-200 p-16 text-center">
        <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
        </svg>
        <h3 className="text-lg font-medium text-gray-900 mb-2">Model Configuration Coming Soon</h3>
        <p className="text-sm text-gray-600">
          Connect your own AI model API keys for enhanced functionality
        </p>
      </div>
    </div>
  );
}
