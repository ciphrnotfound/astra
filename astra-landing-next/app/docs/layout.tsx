import DocsNavbar from '@/components/docs/DocsNavbar';
import DocsSidebar from '@/components/docs/DocsSidebar';
import Footer from '@/components/Footer';

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="min-h-screen bg-[#faf9f6]">
      <DocsNavbar />
      <div className="flex pt-[73px]">
        <DocsSidebar />
        <main className="flex-1">
          {children}
        </main>
      </div>
      <Footer />
    </div>
  );
}
