import { NextResponse } from 'next/server';
import { db } from '@/lib/supabase/server';
import { getSession } from '@/lib/auth/session';

export async function DELETE(
  request: Request,
  { params }: { params: { id: string } }
) {
  try {
    const session = await getSession();
    
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const keyId = parseInt(params.id);

    const { error } = await db
      .from('api_keys')
      .delete()
      .eq('id', keyId)
      .eq('user_id', session.id);

    if (error) {
      console.error('Error deleting API key:', error);
      return NextResponse.json({ error: 'Failed to delete API key' }, { status: 500 });
    }

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Error deleting API key:', error);
    return NextResponse.json({ error: 'Failed to delete API key' }, { status: 500 });
  }
}
