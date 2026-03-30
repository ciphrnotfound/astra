import { NextResponse } from 'next/server';
import { db } from '@/lib/supabase/server';
import { getSession } from '@/lib/auth/session';
import crypto from 'crypto';

export async function GET() {
  try {
    const session = await getSession();
    
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { data: apiKeys, error } = await db
      .from('api_keys')
      .select('id, name, key_prefix, last_used_at, created_at, expires_at, is_active')
      .eq('user_id', session.id)
      .order('created_at', { ascending: false });

    if (error) {
      console.error('Error fetching API keys:', error);
      return NextResponse.json({ error: 'Failed to fetch API keys' }, { status: 500 });
    }

    // Convert snake_case to camelCase
    const formattedKeys = apiKeys?.map(key => ({
      id: key.id,
      name: key.name,
      keyPrefix: key.key_prefix,
      lastUsedAt: key.last_used_at,
      createdAt: key.created_at,
      expiresAt: key.expires_at,
      isActive: key.is_active,
    })) || [];

    return NextResponse.json({ apiKeys: formattedKeys });
  } catch (error) {
    console.error('Error fetching API keys:', error);
    return NextResponse.json({ error: 'Failed to fetch API keys' }, { status: 500 });
  }
}

export async function POST(request: Request) {
  try {
    const session = await getSession();
    
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { name } = await request.json();

    if (!name) {
      return NextResponse.json({ error: 'Name is required' }, { status: 400 });
    }

    // Generate API key: astra_live_[32 random chars]
    const randomBytes = crypto.randomBytes(24);
    const apiKey = `astra_live_${randomBytes.toString('base64url')}`;
    
    // Hash the key for storage
    const keyHash = crypto.createHash('sha256').update(apiKey).digest('hex');
    const keyPrefix = apiKey.substring(0, 16);

    const { error } = await db
      .from('api_keys')
      .insert({
        user_id: session.id,
        name,
        key_hash: keyHash,
        key_prefix: keyPrefix,
      });

    if (error) {
      console.error('Error creating API key:', error);
      return NextResponse.json({ error: 'Failed to create API key' }, { status: 500 });
    }

    return NextResponse.json({ key: apiKey }, { status: 201 });
  } catch (error) {
    console.error('Error creating API key:', error);
    return NextResponse.json({ error: 'Failed to create API key' }, { status: 500 });
  }
}
