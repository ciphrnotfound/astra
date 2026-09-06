import { createBrowserClient } from '@supabase/ssr';
import type { SupabaseClient } from '@supabase/supabase-js';

function requiredPublicEnv(name: 'NEXT_PUBLIC_SUPABASE_URL' | 'NEXT_PUBLIC_SUPABASE_ANON_KEY') {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing ${name}`);
  }
  return value;
}

// Client-side Supabase client factory
export function createClient() {
  return createBrowserClient<any>(
    requiredPublicEnv('NEXT_PUBLIC_SUPABASE_URL'),
    requiredPublicEnv('NEXT_PUBLIC_SUPABASE_ANON_KEY')
  );
}

let browserClient: SupabaseClient<any> | undefined;

function getBrowserClient() {
  browserClient ??= createClient();
  return browserClient;
}

// Database helpers using Supabase client
export const db = {
  from: (table: string) => getBrowserClient().from(table),
};
