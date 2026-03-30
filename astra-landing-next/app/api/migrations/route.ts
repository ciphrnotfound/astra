import { NextResponse } from 'next/server';
import { db } from '@/lib/supabase/server';
import { getSession } from '@/lib/auth/session';

export async function GET(request: Request) {
  try {
    const session = await getSession();
    
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { searchParams } = new URL(request.url);
    const projectId = searchParams.get('projectId');

    let query = db
      .from('migrations')
      .select('*')
      .eq('user_id', session.id)
      .order('created_at', { ascending: false });

    if (projectId) {
      query = query.eq('project_id', parseInt(projectId));
    }

    const { data: migrations, error } = await query;

    if (error) {
      console.error('Error fetching migrations:', error);
      return NextResponse.json({ error: 'Failed to fetch migrations' }, { status: 500 });
    }

    return NextResponse.json({ migrations: migrations || [] });
  } catch (error) {
    console.error('Error fetching migrations:', error);
    return NextResponse.json({ error: 'Failed to fetch migrations' }, { status: 500 });
  }
}

export async function POST(request: Request) {
  try {
    const session = await getSession();
    
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { projectId, sourceLanguage, targetLanguage, totalFiles } = body;

    if (!projectId || !sourceLanguage || !targetLanguage) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    const { data: migration, error } = await db
      .from('migrations')
      .insert({
        project_id: projectId,
        user_id: session.id,
        source_language: sourceLanguage,
        target_language: targetLanguage,
        total_files: totalFiles || 0,
        status: 'pending',
      })
      .select()
      .single();

    if (error) {
      console.error('Error creating migration:', error);
      return NextResponse.json({ error: 'Failed to create migration' }, { status: 500 });
    }

    return NextResponse.json({ migration }, { status: 201 });
  } catch (error) {
    console.error('Error creating migration:', error);
    return NextResponse.json({ error: 'Failed to create migration' }, { status: 500 });
  }
}
