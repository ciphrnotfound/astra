import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Migrations - Astra Dashboard',
  description: 'Migration history and reports',
};

export default function MigrationsPage() {
  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Migrations
        </h1>
        <p className="text-gray-600">All past migrations, status, output files, and reports</p>
      </div>

      <div className="bg-white border border-gray-200 p-16 text-center">
        <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
        </svg>
        <h3 className="text-lg font-medium text-gray-900 mb-2">No migrations yet</h3>
        <p className="text-sm text-gray-600">
          Start your first code migration to see history here
        </p>
      </div>
    </div>
  );
}
