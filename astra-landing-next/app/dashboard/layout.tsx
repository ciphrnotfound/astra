import DashboardSidebar from '@/components/DashboardSidebar';
import SessionRefresh from '@/components/SessionRefresh';
import { requireAuth } from '@/lib/supabase/middleware';

export default async function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  // Require authentication - redirects to /signin if not authenticated
  const user = await requireAuth();

  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <SessionRefresh />
      <DashboardSidebar user={user} />
      
      {/* Main Content with left margin for sidebar */}
      <main className="ml-64 p-8">
        {children}
      </main>
    </div>
  );
}
