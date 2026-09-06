import { createServerClient as createSupabaseServerClient } from '@supabase/ssr';
import { cookies } from 'next/headers';
import { createClient } from '@supabase/supabase-js';
import type { SupabaseClient } from '@supabase/supabase-js';

function requiredEnv(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing ${name}`);
  }
  return value;
}

// Server-side client for authenticated requests (uses cookies)
export async function createServerClient() {
  const cookieStore = await cookies();

  return createSupabaseServerClient(
    requiredEnv('NEXT_PUBLIC_SUPABASE_URL'),
    requiredEnv('NEXT_PUBLIC_SUPABASE_ANON_KEY'),
    {
      cookies: {
        get(name: string) {
          return cookieStore.get(name)?.value;
        },
        set(name: string, value: string, options: any) {
          try {
            cookieStore.set({ name, value, ...options });
          } catch (error) {
            // Handle cookie setting errors in middleware
          }
        },
        remove(name: string, options: any) {
          try {
            cookieStore.set({ name, value: '', ...options });
          } catch (error) {
            // Handle cookie removal errors in middleware
          }
        },
      },
    }
  );
}

let adminClient: SupabaseClient<any> | undefined;

// Create the privileged client only when an admin query is actually made. This
// keeps module imports safe in tests and build tooling while still failing fast
// at the operation boundary when configuration is genuinely required.
function getAdminClient() {
  adminClient ??= createClient<any>(
    requiredEnv('NEXT_PUBLIC_SUPABASE_URL'),
    requiredEnv('SUPABASE_SERVICE_ROLE_KEY'),
    {
      auth: {
        autoRefreshToken: false,
        persistSession: false,
      },
    }
  );

  return adminClient;
}

// Database helpers using Supabase admin client
export const db = {
  from: (table: string) => getAdminClient().from(table),
};
