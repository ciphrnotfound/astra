import { NextResponse } from 'next/server';
import { db } from '@/lib/supabase/server';
import { getSession } from '@/lib/auth/session';

export async function GET() {
  try {
    const session = await getSession();
    
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const userId = session.id;

    // Get total projects
    const { count: totalProjects } = await db
      .from('projects')
      .select('*', { count: 'exact', head: true })
      .eq('user_id', userId);

    // Get total migrations
    const { count: totalMigrations } = await db
      .from('migrations')
      .select('*', { count: 'exact', head: true })
      .eq('user_id', userId);

    // Get completed migrations
    const { count: completedMigrations } = await db
      .from('migrations')
      .select('*', { count: 'exact', head: true })
      .eq('user_id', userId)
      .eq('status', 'completed');

    // Get migrations data for files processed
    const { data: migrationsData } = await db
      .from('migrations')
      .select('files_processed')
      .eq('user_id', userId);

    const totalFilesProcessed = migrationsData?.reduce(
      (sum, m) => sum + (m.files_processed || 0),
      0
    ) || 0;

    return NextResponse.json({
      totalProjects: totalProjects || 0,
      totalMigrations: totalMigrations || 0,
      completedMigrations: completedMigrations || 0,
      filesProcessed: totalFilesProcessed,
    });
  } catch (error) {
    console.error('Error fetching dashboard stats:', error);
    return NextResponse.json(
      { error: 'Failed to fetch stats' },
      { status: 500 }
    );
  }
}
