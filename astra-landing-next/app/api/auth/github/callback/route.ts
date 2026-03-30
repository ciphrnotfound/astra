import { NextResponse } from 'next/server';
import { createServerClient } from '@/lib/supabase/server';

export async function GET(request: Request) {
  const { searchParams, origin } = new URL(request.url);
  const code = searchParams.get('code');
  const next = searchParams.get('next') ?? '/dashboard';

  if (!code) {
    return NextResponse.redirect(`${origin}/signin?error=no_code`);
  }

  try {
    const supabase = await createServerClient();
    
    // Exchange code for session using Supabase Auth
    const { data, error } = await supabase.auth.exchangeCodeForSession(code);

    if (error) {
      console.error('Error exchanging code for session:', error);
      return NextResponse.redirect(`${origin}/signin?error=auth_failed`);
    }

    if (!data.session) {
      return NextResponse.redirect(`${origin}/signin?error=no_session`);
    }

    // Create user profile if it doesn't exist
    const { data: profile } = await supabase
      .from('user_profiles')
      .select('id')
      .eq('id', data.user.id)
      .single();

    if (!profile) {
      // Create profile for new user
      await supabase
        .from('user_profiles')
        .insert({
          id: data.user.id,
          email: data.user.email!,
          name: data.user.user_metadata?.name || data.user.email?.split('@')[0],
          role: 'member',
        });
    }

    // Redirect to dashboard
    return NextResponse.redirect(`${origin}${next}`);
  } catch (error) {
    console.error('GitHub OAuth error:', error);
    return NextResponse.redirect(`${origin}/signin?error=auth_failed`);
  }
}
