'use client';

import { useState } from 'react';
import { Home, FileText, FolderOpen, Settings, LayoutDashboard, PanelLeftClose, PanelLeft, Key } from 'lucide-react';
import Link from 'next/link';

export default function DashboardSidebar() {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isPinned, setIsPinned] = useState(false);

  const menuItems = [
    { icon: Home, label: 'Overview', href: '/dashboard' },
    { icon: FileText, label: 'Migrations', href: '/dashboard/migrations' },
    { icon: FolderOpen, label: 'Projects', href: '/dashboard/projects' },
    { icon: LayoutDashboard, label: 'Analytics', href: '/dashboard/analytics' },
    { icon: Key, label: 'API Keys', href: '/dashboard/api-keys' },
    { icon: Settings, label: 'Settings', href: '/dashboard/settings' },
  ];

  const handleMouseEnter = () => {
    if (!isPinned) {
      setIsExpanded(true);
    }
  };

  const handleMouseLeave = () => {
    if (!isPinned) {
      setIsExpanded(false);
    }
  };

  const togglePin = () => {
    setIsPinned(!isPinned);
    setIsExpanded(!isPinned);
  };

  return (
    <aside
      className={`fixed left-0 top-16 h-[calc(100vh-4rem)] bg-[#faf9f6] border-r border-gray-200 transition-all duration-300 ease-in-out z-40 ${
        isExpanded || isPinned ? 'w-64' : 'w-20'
      }`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <div className="flex flex-col h-full py-4">
        {/* Pin/Unpin Button */}
        <div className="px-3 mb-6 flex justify-end">
          <button
            onClick={togglePin}
            className="relative group overflow-hidden p-1.5 border border-gray-300 text-gray-600 transition-all hover:text-white"
            title={isPinned ? 'Unpin sidebar' : 'Pin sidebar'}
          >
            {isPinned ? (
              <PanelLeftClose className="w-4 h-4 relative z-10" />
            ) : (
              <PanelLeft className="w-4 h-4 relative z-10" />
            )}
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </button>
        </div>

        {/* Menu Items */}
        <nav className="flex-1 px-3 space-y-1">
          {menuItems.map((item) => (
            <Link
              key={item.label}
              href={item.href}
              className="relative group overflow-hidden flex items-center gap-3 px-3 py-2.5 text-gray-600 transition-all hover:text-white"
            >
              <item.icon className="w-5 h-5 flex-shrink-0 relative z-10" />
              <span
                className={`relative z-10 text-sm font-medium whitespace-nowrap transition-all duration-300 ${
                  isExpanded || isPinned ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-4 absolute'
                }`}
              >
                {item.label}
              </span>
              <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
            </Link>
          ))}
        </nav>

        {/* Footer - User Profile */}
        <div className="px-3 pt-4 border-t border-gray-200">
          <div className="relative group overflow-hidden flex items-center gap-3 px-3 py-2.5 cursor-pointer transition-all hover:text-white">
            <div className="w-7 h-7 bg-gray-900 flex items-center justify-center flex-shrink-0 relative z-10">
              <span className="text-white text-xs font-medium">U</span>
            </div>
            <div
              className={`relative z-10 transition-all duration-300 ${
                isExpanded || isPinned ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-4 absolute'
              }`}
            >
              <div className="text-sm font-medium text-gray-900 whitespace-nowrap group-hover:text-white">User</div>
              <div className="text-xs text-gray-600 whitespace-nowrap group-hover:text-gray-300">user@example.com</div>
            </div>
            <div className="absolute inset-0 bg-gray-900 transform translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out" />
          </div>
        </div>
      </div>
    </aside>
  );
}
