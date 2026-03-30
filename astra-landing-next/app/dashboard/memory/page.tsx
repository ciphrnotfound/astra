import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Memory - Astra Dashboard',
  description: 'Codebase memory timeline and snapshots',
};

export default function MemoryPage() {
  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Codebase Memory
        </h1>
        <p className="text-gray-600">Every indexed snapshot, what changed, when and why</p>
      </div>

      <div className="bg-white border border-gray-200 p-16 text-center">
        <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <h3 className="text-lg font-medium text-gray-900 mb-2">No snapshots yet</h3>
        <p className="text-sm text-gray-600">
          Index your codebase to start tracking changes over time
        </p>
      </div>
    </div>
  );
}
