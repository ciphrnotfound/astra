import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Semantic Graph - Astra Dashboard',
  description: 'Interactive dependency visualization of your codebase',
};

export default function GraphPage() {
  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Semantic Cartographer
        </h1>
        <p className="text-gray-600">Interactive dependency visualization of your codebase</p>
      </div>

      <div className="bg-white border border-gray-200 p-16 text-center">
        <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
        </svg>
        <h3 className="text-lg font-medium text-gray-900 mb-2">Dependency Graph Coming Soon</h3>
        <p className="text-sm text-gray-600">
          Visualize your codebase structure and dependencies
        </p>
      </div>
    </div>
  );
}
