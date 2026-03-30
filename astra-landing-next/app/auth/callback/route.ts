import { NextResponse } from 'next/server';
import { createServerClient } from '@/lib/supabase/server';

export async function GET(request: Request) {
  const { searchParams, origin } = new URL(request.url);
  const code = searchParams.get('code');
  const next = searchParams.get('next') ?? '/dashboard';

  if (code) {
    try {
      const supabase = await createServerClient();
      const { data, error } = await supabase.auth.exchangeCodeForSession(code);
      
      if (error) {
        console.error('Error exchanging code for session:', error);
        return NextResponse.redirect(`${origin}/signin?error=session_exchange_failed`);
      }

      if (!data.session) {
        console.error('No session returned after code exchange');
        return NextResponse.redirect(`${origin}/signin?error=no_session`);
      }

      // Get the user to create profile if needed
      const { data: { user } } = await supabase.auth.getUser();
      
      if (user) {
        // Check if profile exists
        const { data: profile, error: profileError } = await supabase
          .from('user_profiles')
          .select('id')
          .eq('id', user.id)
          .single();

        if (!profile && !profileError) {
          // Create profile for new user
          const { error: insertError } = await supabase
            .from('user_profiles')
            .insert({
              id: user.id,
              email: user.email!,
              name: user.user_metadata?.name || user.email?.split('@')[0],
              role: 'member',
            });

          if (insertError) {
            console.error('Error creating user profile:', insertError);
            // Continue anyway - profile creation is not critical for auth
          }
        }
      }
      
      return NextResponse.redirect(`${origin}${next}`);
    } catch (err) {
      console.error('Callback error:', err);
      return NextResponse.redirect(`${origin}/signin?error=callback_exception`);
    }
  }

  // No code provided
  console.error('No code in callback URL');
  return NextResponse.redirect(`${origin}/signin?error=no_code`);
}

