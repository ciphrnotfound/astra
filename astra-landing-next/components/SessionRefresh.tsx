'use client';

import { useEffect } from 'react';
import { createClient } from '@/lib/supabase/client';
import { useRouter } from 'next/navigation';

/**
 * SessionRefresh Component
 * Handles automatic session refresh and expired session redirects
 * Should be included in the dashboard layout
 */
export default function SessionRefresh() {
  const router = useRouter();
  const supabase = createClient();

  useEffect(() => {
    // Listen for auth state changes
    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((event, session) => {
      if (event === 'SIGNED_OUT' || event === 'USER_DELETED') {
        // Redirect to sign-in page when user signs out
        router.push('/signin');
      } else if (event === 'TOKEN_REFRESHED') {
        // Refresh the page to update server-side session
        router.refresh();
      } else if (event === 'SIGNED_IN') {
        // Refresh the page when user signs in
        router.refresh();
      }
    });

    // Check session expiry every minute
    const checkSession = setInterval(async () => {
      const { data: { session } } = await supabase.auth.getSession();
      
      if (!session) {
        // No session - redirect to sign-in
        router.push('/signin');
        return;
      }

      const expiresAt = session.expires_at;
      if (!expiresAt) return;

      const now = Math.floor(Date.now() / 1000);
      const fiveMinutes = 5 * 60;

      // Refresh session if expiring within 5 minutes
      if (expiresAt - now < fiveMinutes) {
        const { error } = await supabase.auth.refreshSession();
        if (error) {
          console.error('Session refresh failed:', error);
          router.push('/signin');
        }
      }
    }, 60000); // Check every minute

    return () => {
      subscription.unsubscribe();
      clearInterval(checkSession);
    };
  }, [router, supabase]);

  return null; // This component doesn't render anything
}
