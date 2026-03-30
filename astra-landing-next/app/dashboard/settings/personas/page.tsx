import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Personas - Settings - Astra Dashboard',
  description: 'Switch AI personas and vibes',
};

export default function PersonasPage() {
  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          AI Personas
        </h1>
        <p className="text-gray-600">Switch vibes — Architect, Pidgin, Brutal, Doge etc</p>
      </div>

      <div className="bg-white border border-gray-200 p-16 text-center">
        <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <h3 className="text-lg font-medium text-gray-900 mb-2">Personas Coming Soon</h3>
        <p className="text-sm text-gray-600">
          Choose your AI assistant's personality and communication style
        </p>
      </div>
    </div>
  );
}
