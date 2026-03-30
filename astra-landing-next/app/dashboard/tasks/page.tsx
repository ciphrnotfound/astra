import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Tasks - Astra Dashboard',
  description: 'Team task manager with velocity metrics',
};

export default function TasksPage() {
  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Team Tasks
        </h1>
        <p className="text-gray-600">Assigned tasks, velocity metrics, developer breakdown</p>
      </div>

      <div className="bg-white border border-gray-200 p-16 text-center">
        <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
        </svg>
        <h3 className="text-lg font-medium text-gray-900 mb-2">No tasks yet</h3>
        <p className="text-sm text-gray-600">
          Create tasks to track team progress and velocity
        </p>
      </div>
    </div>
  );
}
