import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Security - Astra Dashboard',
  description: 'Security Hunter results and vulnerability tracking',
};

export default function SecurityPage() {
  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Security Hunter
        </h1>
        <p className="text-gray-600">All vulnerabilities ranked by severity, status (fixed/open)</p>
      </div>

      <div className="bg-white border border-gray-200 p-16 text-center">
        <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
        </svg>
        <h3 className="text-lg font-medium text-gray-900 mb-2">No security scans yet</h3>
        <p className="text-sm text-gray-600">
          Run your first security scan to identify vulnerabilities
        </p>
      </div>
    </div>
  );
}
