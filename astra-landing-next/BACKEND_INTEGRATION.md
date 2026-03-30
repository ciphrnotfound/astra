# Astra Backend Integration Guide

## Overview

The Astra landing page now has a complete backend powered by **Supabase** (PostgreSQL) with **Drizzle ORM** for type-safe database operations.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Next.js Frontend                         │
│  (React Components, Pages, Client-side Logic)               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   API Routes (Next.js)                       │
│  /api/dashboard/stats  │  /api/projects  │  /api/migrations │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Drizzle ORM Layer                         │
│         (Type-safe queries, schema validation)              │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Supabase PostgreSQL Database                    │
│  (Users, Projects, Migrations, Analytics, Teams)            │
└─────────────────────────────────────────────────────────────┘
```

## Database Schema

### User Management
- **users** - Authentication and user profiles
- **teams** - Organization/team management
- **team_members** - User-team relationships
- **invitations** - Team invitation system
- **activity_logs** - Audit trail

### Astra Features
- **projects** - User code projects
  - Links to users
  - Stores repository info, language, metadata
  
- **migrations** - Cross-language migration tracking
  - Links to projects and users
  - Tracks source/target languages
  - Status tracking (pending, in_progress, completed, failed)
  - Files processed count
  
- **codebase_analytics** - Project health metrics
  - Lines of code
  - Technical debt score
  - Test coverage
  - Security score

## API Endpoints

### Dashboard Stats
**GET** `/api/dashboard/stats`

Returns aggregated statistics for the logged-in user:
```json
{
  "totalProjects": 5,
  "totalMigrations": 12,
  "completedMigrations": 10,
  "filesProcessed": 247
}
```

### Projects
**GET** `/api/projects`
- Returns all projects for the logged-in user
- Ordered by creation date (newest first)

**POST** `/api/projects`
- Creates a new project
- Required: `name`
- Optional: `description`, `repositoryUrl`, `language`

```json
{
  "name": "My Awesome Project",
  "description": "A cool project",
  "repositoryUrl": "https://github.com/user/repo",
  "language": "TypeScript"
}
```

### Migrations
**GET** `/api/migrations?projectId=123`
- Returns migrations for the logged-in user
- Optional query param: `projectId` to filter by project

**POST** `/api/migrations`
- Creates a new migration
- Required: `projectId`, `sourceLanguage`, `targetLanguage`
- Optional: `totalFiles`

```json
{
  "projectId": 1,
  "sourceLanguage": "Python",
  "targetLanguage": "Go",
  "totalFiles": 25
}
```

## Authentication Flow

1. User signs up/signs in via `/signin` or `/signup`
2. Session is created using JWT (jose library)
3. Session stored in HTTP-only cookie
4. API routes check session via `getSession()`
5. Unauthorized requests return 401

## Using the API in Components

### Client-side (React Components)

```typescript
'use client';

import { useEffect, useState } from 'react';

export default function DashboardStats() {
  const [stats, setStats] = useState(null);

  useEffect(() => {
    fetch('/api/dashboard/stats')
      .then(res => res.json())
      .then(data => setStats(data));
  }, []);

  if (!stats) return <div>Loading...</div>;

  return (
    <div>
      <p>Projects: {stats.totalProjects}</p>
      <p>Migrations: {stats.totalMigrations}</p>
    </div>
  );
}
```

### Server-side (Server Components)

```typescript
import { db } from '@/lib/db/drizzle';
import { projects } from '@/lib/db/schema';
import { getSession } from '@/lib/auth/session';
import { eq } from 'drizzle-orm';

export default async function ProjectsPage() {
  const session = await getSession();
  
  if (!session) {
    redirect('/signin');
  }

  const userProjects = await db
    .select()
    .from(projects)
    .where(eq(projects.userId, session.id));

  return (
    <div>
      {userProjects.map(project => (
        <div key={project.id}>{project.name}</div>
      ))}
    </div>
  );
}
```

## Database Queries with Drizzle

### Basic Queries

```typescript
import { db } from '@/lib/db/drizzle';
import { projects, migrations } from '@/lib/db/schema';
import { eq, desc, and } from 'drizzle-orm';

// Select all projects for a user
const userProjects = await db
  .select()
  .from(projects)
  .where(eq(projects.userId, userId));

// Insert a new project
const [newProject] = await db
  .insert(projects)
  .values({
    userId: 1,
    name: 'My Project',
    description: 'Description',
  })
  .returning();

// Update a project
await db
  .update(projects)
  .set({ name: 'Updated Name' })
  .where(eq(projects.id, projectId));

// Delete a project
await db
  .delete(projects)
  .where(eq(projects.id, projectId));
```

### Complex Queries

```typescript
// Join projects with migrations
const projectsWithMigrations = await db
  .select()
  .from(projects)
  .leftJoin(migrations, eq(projects.id, migrations.projectId))
  .where(eq(projects.userId, userId));

// Count migrations by status
const migrationStats = await db
  .select({
    status: migrations.status,
    count: count(),
  })
  .from(migrations)
  .where(eq(migrations.userId, userId))
  .groupBy(migrations.status);
```

## Environment Variables

Required for backend to work:

```env
# Database
POSTGRES_URL=postgresql://...

# Supabase
NEXT_PUBLIC_SUPABASE_URL=https://...
NEXT_PUBLIC_SUPABASE_ANON_KEY=...
SUPABASE_SERVICE_ROLE_KEY=...

# Auth
AUTH_SECRET=...

# Base URL
BASE_URL=http://localhost:3000
```

## Development Workflow

1. **Make schema changes** in `lib/db/schema.ts`
2. **Generate migration**: `pnpm db:generate`
3. **Apply migration**: `pnpm db:migrate`
4. **View database**: `pnpm db:studio` (opens Drizzle Studio)

## Testing

### Manual Testing
1. Start dev server: `pnpm dev`
2. Sign up for an account
3. Use browser DevTools Network tab to inspect API calls
4. Check Supabase dashboard for data

### API Testing with curl

```bash
# Get dashboard stats (requires auth cookie)
curl http://localhost:3000/api/dashboard/stats \
  -H "Cookie: session=..."

# Create a project
curl -X POST http://localhost:3000/api/projects \
  -H "Content-Type: application/json" \
  -H "Cookie: session=..." \
  -d '{"name":"Test Project","language":"TypeScript"}'
```

## Deployment

### Vercel Deployment

1. Push code to GitHub
2. Import project in Vercel
3. Add environment variables in Vercel dashboard
4. Deploy!

### Environment Variables in Production

Make sure to set all required env vars in Vercel:
- `POSTGRES_URL`
- `NEXT_PUBLIC_SUPABASE_URL`
- `NEXT_PUBLIC_SUPABASE_ANON_KEY`
- `SUPABASE_SERVICE_ROLE_KEY`
- `AUTH_SECRET`
- `BASE_URL` (your production URL)

## Security Best Practices

1. **Never expose service role key** to client
2. **Always validate user sessions** in API routes
3. **Use Row Level Security (RLS)** in Supabase for extra protection
4. **Sanitize user inputs** before database queries
5. **Use HTTPS** in production
6. **Rotate secrets** regularly

## Monitoring

### Supabase Dashboard
- Monitor database performance
- View query logs
- Check auth activity
- Monitor storage usage

### Application Logs
- Check Vercel logs for API errors
- Monitor error rates
- Track slow queries

## Next Steps

1. ✅ Set up Supabase project
2. ✅ Configure environment variables
3. ✅ Run database migrations
4. 🔄 Connect dashboard to real data
5. 🔄 Add real-time features with Supabase subscriptions
6. 🔄 Implement file upload for project files
7. 🔄 Add webhook for Astra CLI integration
8. 🔄 Set up monitoring and alerts

## Troubleshooting

### "Unauthorized" errors
- Check if user is logged in
- Verify session cookie is being sent
- Check `AUTH_SECRET` is set correctly

### Database connection errors
- Verify `POSTGRES_URL` is correct
- Check Supabase project is running
- Ensure IP is whitelisted (if applicable)

### Migration errors
- Delete `lib/db/migrations` and regenerate
- Check for syntax errors in schema
- Verify database is accessible

## Resources

- [Drizzle ORM Docs](https://orm.drizzle.team)
- [Supabase Docs](https://supabase.com/docs)
- [Next.js API Routes](https://nextjs.org/docs/app/building-your-application/routing/route-handlers)
