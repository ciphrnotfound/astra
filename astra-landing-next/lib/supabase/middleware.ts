// Authentication Middleware for Astra Landing Application
// Handles session validation and redirects for protected routes

import { createServerClient } from './server';
import { redirect } from 'next/navigation';
import type { User } from '@supabase/supabase-js';

function isNextDynamicServerUsage(error: unknown): boolean {
  return (
    typeof error === 'object' &&
    error !== null &&
    'digest' in error &&
    (error as { digest?: unknown }).digest === 'DYNAMIC_SERVER_USAGE'
  );
}

/**
 * Require authentication for a route
 * Redirects to sign-in page if user is not authenticated
 * Returns the authenticated user if successful
 */
export async function requireAuth(): Promise<User> {
  let session: { user: User } | null = null;
  let error: unknown = null;

  try {
    const supabase = await createServerClient();
    const result = await supabase.auth.getSession();
    session = result.data.session;
    error = result.error;
  } catch (cause) {
    // Next uses this sentinel during static analysis of cookie-based routes;
    // swallowing it would make the route appear static and hide real bugs.
    if (isNextDynamicServerUsage(cause)) {
      throw cause;
    }
    // Missing/invalid deployment configuration must not turn a protected
    // route into a server error.  Redirect to the public auth surface and log
    // the actionable cause for operators.
    console.error('Auth configuration unavailable:', cause);
    redirect('/signin?error=auth_unavailable');
  }

  if (error || !session) {
    redirect('/signin');
  }
  
  return session.user;
}

/**
 * Get the current user if authenticated, null otherwise
 * Does not redirect - useful for optional authentication
 */
export async function getCurrentUser(): Promise<User | null> {
  try {
    const supabase = await createServerClient();
    const { data: { session } } = await supabase.auth.getSession();
    return session?.user || null;
  } catch (cause) {
    if (isNextDynamicServerUsage(cause)) {
      throw cause;
    }
    console.error('Unable to read the current auth session:', cause);
    return null;
  }
}

/**
 * Check if user is authenticated
 * Returns boolean without redirecting
 */
export async function isAuthenticated(): Promise<boolean> {
  const supabase = await createServerClient();
  
  const { data: { session } } = await supabase.auth.getSession();
  
  return !!session;
}

/**
 * Require specific role for a route
 * Redirects to dashboard if user doesn't have required role
 */
export async function requireRole(role: string): Promise<User> {
  const user = await requireAuth();
  
  // Get user role from database
  const supabase = await createServerClient();
  const { data: userData } = await supabase
    .from('user_profiles')
    .select('role')
    .eq('id', user.id)
    .single();
  
  if (!userData || userData.role !== role) {
    redirect('/dashboard');
  }
  
  return user;
}

/**
 * Refresh the current session
 * Returns the refreshed session or null if refresh fails
 */
export async function refreshSession() {
  const supabase = await createServerClient();
  
  const { data, error } = await supabase.auth.refreshSession();
  
  if (error) {
    console.error('Session refresh failed:', error);
    return null;
  }
  
  return data.session;
}

/**
 * Sign out the current user
 * Clears session and redirects to landing page
 */
export async function signOut() {
  const supabase = await createServerClient();
  
  await supabase.auth.signOut();
  redirect('/');
}

/**
 * Get user ID from authenticated session
 * Throws error if not authenticated
 */
export async function getUserId(): Promise<string> {
  const user = await requireAuth();
  return user.id;
}

/**
 * Check if session is expiring soon (within 5 minutes)
 * Returns true if session needs refresh
 */
export async function isSessionExpiringSoon(): Promise<boolean> {
  const supabase = await createServerClient();
  
  const { data: { session } } = await supabase.auth.getSession();
  
  if (!session) return false;
  
  const expiresAt = session.expires_at;
  if (!expiresAt) return false;
  
  const now = Math.floor(Date.now() / 1000);
  const fiveMinutes = 5 * 60;
  
  return (expiresAt - now) < fiveMinutes;
}
