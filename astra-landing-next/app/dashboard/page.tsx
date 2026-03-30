import { Metadata } from 'next';
import Link from 'next/link';
import { getCurrentUser } from '@/lib/supabase/middleware';

export const metadata: Metadata = {
  title: 'Dashboard - Astra',
  description: 'Manage your Astra projects and migrations.',
};

export default async function DashboardPage() {
  const user = await getCurrentUser();
  const userName = user?.user_metadata?.name || user?.email?.split('@')[0] || 'there';

  return (
    <div className="max-w-7xl mx-auto px-6">
      {/* Header */}
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Welcome back, {userName}
        </h1>
        <p className="text-gray-600">Here's what's happening with your codebase</p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        <div className="bg-white border border-gray-200 p-6">
          <div className="text-3xl font-medium text-gray-900 mb-2" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            0
          </div>
          <div className="text-sm text-gray-600">Total Migrations</div>
        </div>
        <div className="bg-white border border-gray-200 p-6">
          <div className="text-3xl font-medium text-gray-900 mb-2" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            0
          </div>
          <div className="text-sm text-gray-600">Files Processed</div>
        </div>
        <div className="bg-white border border-gray-200 p-6">
          <div className="text-3xl font-medium text-gray-900 mb-2" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            0
          </div>
          <div className="text-sm text-gray-600">Active Projects</div>
        </div>
      </div>

      {/* Recent Migrations */}
      <div className="mb-12">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-medium text-gray-900 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Recent Migrations
          </h2>
          <button className="relative group overflow-hidden bg-gray-900 text-white px-4 py-2 text-sm font-medium transition-all hover:shadow-lg">
            <span className="relative z-10">New Migration</span>
            <div className="absolute inset-0 bg-gray-800 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300 ease-out origin-left" />
          </button>
        </div>

        <div className="bg-white border border-gray-200">
          <div className="p-12 text-center">
            <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <h3 className="text-lg font-medium text-gray-900 mb-2">No migrations yet</h3>
            <p className="text-sm text-gray-600 mb-6">
              Get started by running the Astra CLI in your project
            </p>
            <Link
              href="/docs"
              className="relative group overflow-hidden border border-gray-900 text-gray-900 px-6 py-3 text-sm font-medium transition-all hover:text-white inline-block"
            >
              <span className="relative z-10">View Documentation</span>
              <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
            </Link>
          </div>
        </div>
      </div>

      {/* Projects */}
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-medium text-gray-900 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
            Projects
          </h2>
          <button className="relative group overflow-hidden border border-gray-900 text-gray-900 px-4 py-2 text-sm font-medium transition-all hover:text-white">
            <span className="relative z-10">New Project</span>
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </button>
        </div>

        <div className="bg-white border border-gray-200">
          <div className="p-12 text-center">
            <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <h3 className="text-lg font-medium text-gray-900 mb-2">No projects yet</h3>
            <p className="text-sm text-gray-600">
              Projects will appear here once you sync data from the CLI
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
