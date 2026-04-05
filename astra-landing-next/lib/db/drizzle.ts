import { drizzle } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';
import * as schema from './schema';
import dotenv from 'dotenv';

dotenv.config();

// Make POSTGRES_URL optional - using Supabase client instead
const postgresUrl = process.env.POSTGRES_URL || 'postgresql://placeholder';
export const client = postgres(postgresUrl);
export const db = drizzle(client, { schema });
