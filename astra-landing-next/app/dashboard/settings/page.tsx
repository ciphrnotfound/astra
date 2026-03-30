import { Metadata } from 'next';
import Link from 'next/link';
import { User, Key, Users, Puzzle } from 'lucide-react';

export const metadata: Metadata = {
  title: 'Settings - Astra Dashboard',
  description: 'Configure your Astra dashboard',
};

export default function SettingsPage() {
  const settingsCategories = [
    {
      icon: User,
      title: 'Personas',
      description: 'Switch vibes — Architect, Pidgin, Brutal, Doge etc',
      href: '/dashboard/settings/personas',
    },
    {
      icon: Key,
      title: 'Models',
      description: 'BYOK — add your own API keys (Groq, OpenAI, Gemini, Ollama)',
      href: '/dashboard/settings/models',
    },
    {
      icon: Users,
      title: 'Team',
      description: 'Team members, permissions, Supabase sync config',
      href: '/dashboard/settings/team',
    },
    {
      icon: Puzzle,
      title: 'Integrations',
      description: 'MCP, Cursor, VSCode, Git hooks config',
      href: '/dashboard/settings/integrations',
    },
  ];

  return (
    <div className="max-w-7xl mx-auto px-6">
      <div className="mb-12">
        <h1 className="text-4xl font-medium text-gray-900 mb-2 tracking-tight" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
          Settings
        </h1>
        <p className="text-gray-600">Configure your Astra dashboard</p>
      </div>

      <div className="grid md:grid-cols-2 gap-6">
        {settingsCategories.map((category) => (
          <Link
            key={category.title}
            href={category.href}
            className="bg-white border border-gray-200 p-8 transition-all duration-300 hover:border-gray-900 group"
          >
            <div className="flex items-start gap-4">
              <div className="w-12 h-12 border border-gray-900 bg-white flex items-center justify-center shrink-0">
                <category.icon className="w-6 h-6 text-gray-900" />
              </div>
              <div className="flex-1">
                <h3 className="text-xl font-medium text-gray-900 mb-2 group-hover:underline" style={{ fontFamily: 'var(--font-cabinet-grotesk)' }}>
                  {category.title}
                </h3>
                <p className="text-gray-600 text-sm">
                  {category.description}
                </p>
              </div>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
