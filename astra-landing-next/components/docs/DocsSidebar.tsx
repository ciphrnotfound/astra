'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

const sections = [
  {
    title: 'Getting Started',
    items: [
      { title: 'Introduction', href: '/docs' },
      { title: 'Installation', href: '/docs/installation' },
      { title: 'Quick Start', href: '/docs/quick-start' },
    ],
  },
  {
    title: 'Core Concepts',
    items: [
      { title: 'How It Works', href: '/docs/how-it-works' },
      { title: 'Supported Languages', href: '/docs/supported-languages' },
      { title: 'Configuration', href: '/docs/configuration' },
    ],
  },
  {
    title: 'Commands',
    items: [
      { title: 'migrate', href: '/docs/commands/migrate' },
      { title: 'analyze', href: '/docs/commands/analyze' },
      { title: 'validate', href: '/docs/commands/validate' },
    ],
  },
  {
    title: 'Advanced',
    items: [
      { title: 'Custom Rules', href: '/docs/advanced/custom-rules' },
      { title: 'Plugins', href: '/docs/advanced/plugins' },
      { title: 'API Reference', href: '/docs/advanced/api-reference' },
    ],
  },
];

export default function DocsSidebar() {
  const pathname = usePathname();

  return (
    <aside className="w-64 border-r border-gray-200 bg-white sticky top-[73px] h-[calc(100vh-73px)] overflow-y-auto">
      <div className="p-6">
        <div className="mb-8">
          <h2 className="text-sm font-medium text-gray-900 mb-2">Documentation</h2>
          <p className="text-xs text-gray-600">Everything you need to know about Astra</p>
        </div>

        <nav className="space-y-8">
          {sections.map((section) => (
            <div key={section.title}>
              <h3 className="text-xs font-medium text-gray-900 uppercase tracking-wider mb-3">
                {section.title}
              </h3>
              <ul className="space-y-2">
                {section.items.map((item) => {
                  const isActive = pathname === item.href;
                  return (
                    <li key={item.href}>
                      <Link
                        href={item.href}
                        className={`block text-sm transition-colors py-1 ${
                          isActive
                            ? 'text-gray-900 font-medium'
                            : 'text-gray-600 hover:text-gray-900'
                        }`}
                      >
                        {item.title}
                      </Link>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </nav>
      </div>
    </aside>
  );
}
