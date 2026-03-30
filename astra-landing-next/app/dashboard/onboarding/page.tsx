import { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Onboarding - Astra Dashboard',
  description: 'Learning phases for junior devs with progress tracking',
};

export default function OnboardingPage() {
  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Developer Onboarding
        </h1>
        <p className="text-gray-600">Learning phases, progress tracking, unlocked levels, AI review scores</p>
      </div>

      <div className="bg-white border border-gray-200 p-16 text-center">
        <svg className="w-16 h-16 text-gray-300 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
        </svg>
        <h3 className="text-lg font-medium text-gray-900 mb-2">Onboarding Program Coming Soon</h3>
        <p className="text-sm text-gray-600">
          Structured learning paths for new team members
        </p>
      </div>
    </div>
  );
}
