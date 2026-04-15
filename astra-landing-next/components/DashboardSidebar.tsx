'use client';

import { Home, Activity, GitBranch, Shield, CheckSquare, Database, BookOpen, Settings, Key, LogOut } from 'lucide-react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import type { User } from '@supabase/supabase-js';
import { createClient } from '@/lib/supabase/client';

interface DashboardSidebarProps {
  user: User;
}

export default function DashboardSidebar({ user }: DashboardSidebarProps) {
  const pathname = usePathname();
  const router = useRouter();

  const menuItems = [
    { icon: Home, label: 'Overview', href: '/dashboard' },
    { icon: Activity, label: 'Health', href: '/dashboard/health' },
    { icon: GitBranch, label: 'Graph', href: '/dashboard/graph' },
    { icon: Database, label: 'Memory', href: '/dashboard/memory' },
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

  const getDisplayName = () => {
    return user.user_metadata?.name || user.email?.split('@')[0] || 'User';
  };

  return (
    <aside className="fixed left-0 top-0 h-screen w-64 bg-white border-r border-gray-200 flex flex-col z-50">
      {/* Logo */}
      <div className="h-16 flex items-center px-6 border-b border-gray-100">
        <Link href="/" className="flex items-center gap-3">
          <div className="w-8 h-8 flex items-center justify-center shrink-0">
            <img src="/astra-logo-2.png" alt="Astra Logo" className="w-full h-full object-contain" />
          </div>
          <span className="text-sm font-mono text-gray-900 tracking-wide">ASTRA</span>
        </Link>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-4 overflow-y-auto">
        <div className="space-y-1">
          {menuItems.map((item) => (
            <Link
              key={item.label}
              href={item.href}
              className={`flex items-center gap-3 px-3 py-2 text-sm font-medium transition-all ${
                isActive(item.href)
                  ? 'bg-gray-100 text-gray-900'
                  : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
              }`}
            >
              <item.icon className="w-4 h-4 shrink-0" />
              <span>{item.label}</span>
            </Link>
          ))}
        </div>
      </nav>

      {/* User Section */}
      <div className="border-t border-gray-100 p-3">
        <div className="flex items-center gap-3 px-3 py-2 mb-2">
          <div className="w-8 h-8 bg-gray-900 flex items-center justify-center shrink-0">
            <span className="text-white text-xs font-medium">{getUserInitials()}</span>
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-gray-900 truncate">{getDisplayName()}</p>
            <p className="text-xs text-gray-500 truncate">{user.email}</p>
          </div>
        </div>
        <button
          onClick={handleSignOut}
          className="w-full flex items-center gap-3 px-3 py-2 text-sm font-medium text-gray-600 hover:bg-gray-50 hover:text-gray-900 transition-all"
        >
          <LogOut className="w-4 h-4" />
          <span>Sign out</span>
        </button>
      </div>
    </aside>
  );
}
