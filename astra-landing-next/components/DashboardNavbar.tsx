'use client';

import { Home, Activity, GitBranch, Shield, CheckSquare, Database, BookOpen, Settings, Key, LogOut } from 'lucide-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import type { User } from '@supabase/supabase-js';
import { createClient } from '@/lib/supabase/client';

interface DashboardNavbarProps {
  user: User;
}

export default function DashboardNavbar({ user }: DashboardNavbarProps) {
  const pathname = usePathname();
  const router = useRouter();

  const menuItems = [
    { icon: Home, label: 'Overview', href: '/dashboard' },
    { icon: Activity, label: 'Health', href: '/dashboard/health' },
    { icon: GitBranch, label: 'Graph', href: '/dashboard/graph' },
    { icon: Database, label: 'Migrations', href: '/dashboard/migrations' },
    { icon: Shield, label: 'Security', href: '/dashboard/security' },
    { icon: CheckSquare, label: 'Tasks', href: '/dashboard/tasks' },
    { icon: Database, label: 'Memory', href: '/dashboard/memory' },
    { icon: BookOpen, label: 'Onboarding', href: '/dashboard/onboarding' },
    { icon: Key, label: 'API Keys', href: '/dashboard/api-keys' },
    { icon: Settings, label: 'Settings', href: '/dashboard/settings' },
  ];

  const isActive = (href: string) => {
    if (href === '/dashboard') {
      return pathname === href;
    }
    return pathname?.startsWith(href);
  };

  const handleSignOut = async () => {
    const supabase = createClient();
    await supabase.auth.signOut();
    router.push('/');
    router.refresh();
  };

  // Get user initials for avatar
  const getUserInitials = () => {
    if (user.user_metadata?.name) {
      return user.user_metadata.name
        .split(' ')
        .map((n: string) => n[0])
        .join('')
        .toUpperCase()
        .slice(0, 2);
    }
    return user.email?.charAt(0).toUpperCase() || 'U';
  };

  // Get display name
  const getDisplayName = () => {
    return user.user_metadata?.name || user.email?.split('@')[0] || 'User';
  };

  return (
    <>
      {/* Main Top Navbar */}
      <nav className="fixed top-0 left-0 right-0 z-50 bg-[#faf9f6]/80 backdrop-blur-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-3">
            <div className="w-8 h-8 flex items-center justify-center shrink-0">
              <img src="/astra-logo-2.png" alt="Astra Logo" className="w-full h-full object-contain" />
            </div>
          </Link>

          {/* User Menu */}
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 px-3 py-1.5 border border-gray-200">
              <div className="w-6 h-6 bg-gray-900 flex items-center justify-center">
                <span className="text-white text-xs font-medium">{getUserInitials()}</span>
              </div>
              <span className="text-sm text-gray-900">{getDisplayName()}</span>
            </div>
            <button
              onClick={handleSignOut}
              className="relative group overflow-hidden p-2 border border-gray-300 text-gray-600 transition-all hover:text-white"
              title="Sign out"
            >
              <LogOut className="w-4 h-4 relative z-10" />
              <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
            </button>
          </div>
        </div>
      </nav>

      {/* Dashboard Sub-Navigation */}
      <nav className="fixed top-16 left-0 right-0 z-40 bg-[#faf9f6]/80 backdrop-blur-sm border-b border-gray-200 overflow-x-auto">
        <div className="max-w-7xl mx-auto px-6">
          <div className="flex items-center h-14">
            {menuItems.map((item) => (
              <Link
                key={item.label}
                href={item.href}
                className={`relative group flex items-center gap-2 px-4 py-2 text-sm font-medium transition-all whitespace-nowrap ${
                  isActive(item.href)
                    ? 'text-gray-900'
                    : 'text-gray-600 hover:text-gray-900'
                }`}
              >
                <item.icon className="w-4 h-4" />
                <span>{item.label}</span>
                {isActive(item.href) && (
                  <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-gray-900"></span>
                )}
              </Link>
            ))}
          </div>
        </div>
      </nav>
    </>
  );
}
