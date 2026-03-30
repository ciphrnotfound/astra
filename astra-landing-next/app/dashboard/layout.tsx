import DashboardNavbar from '@/components/DashboardNavbar';
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
      <DashboardNavbar user={user} />
      
      {/* Main Content */}
      <main className="pt-32">
        {children}
      </main>
    </div>
  );
}
