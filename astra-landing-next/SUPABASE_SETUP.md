# Supabase Setup Guide for Astra Landing Page

## Prerequisites
- A Supabase account (sign up at https://supabase.com)
- Node.js and npm/pnpm installed

## Step 1: Create a Supabase Project

1. Go to https://supabase.com/dashboard
2. Click "New Project"
3. Fill in:
   - **Name**: astra-landing (or your preferred name)
   - **Database Password**: Create a strong password (save this!)
   - **Region**: Choose closest to your users
4. Click "Create new project" and wait for setup to complete

## Step 2: Get Your Credentials

Once your project is ready:

1. Go to **Project Settings** (gear icon in sidebar)
2. Navigate to **API** section
3. Copy these values:
   - **Project URL** → `NEXT_PUBLIC_SUPABASE_URL`
   - **anon public** key → `NEXT_PUBLIC_SUPABASE_ANON_KEY`
   - **service_role** key → `SUPABASE_SERVICE_ROLE_KEY` (keep this secret!)

4. Navigate to **Database** section
5. Scroll to **Connection string** → **URI**
6. Copy the connection string → `POSTGRES_URL`
   - Replace `[YOUR-PASSWORD]` with your database password

## Step 3: Install Dependencies

```bash
cd astra-landing-next
pnpm install @supabase/supabase-js
```

## Step 4: Configure Environment Variables

1. Create `.env.local` file in the root:

```bash
cp .env.local.example .env.local
```

2. Fill in your Supabase credentials:

```env
# Database (Supabase PostgreSQL)
POSTGRES_URL=postgresql://postgres:[YOUR-PASSWORD]@db.[YOUR-PROJECT-REF].supabase.co:5432/postgres

# Supabase
NEXT_PUBLIC_SUPABASE_URL=https://[YOUR-PROJECT-REF].supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key

# Authentication
AUTH_SECRET=your-random-secret-key-here

# Base URL
BASE_URL=http://localhost:3000
NEXT_PUBLIC_BASE_URL=http://localhost:3000
```

3. Generate a random `AUTH_SECRET`:
```bash
openssl rand -base64 32
```

## Step 5: Set Up Database Schema

1. Generate migration files:
```bash
pnpm db:generate
```

2. Run migrations to create tables:
```bash
pnpm db:migrate
```

3. (Optional) Seed the database with sample data:
```bash
pnpm db:seed
```

## Step 6: Verify Setup

1. Start the development server:
```bash
pnpm dev
```

2. Open http://localhost:3000
3. Try signing up for an account
4. Check your Supabase dashboard → **Table Editor** to see the data

## Database Schema

The following tables will be created:

### Core Tables
- **users** - User accounts with authentication
- **teams** - Team/organization management
- **team_members** - User-team relationships
- **activity_logs** - Audit trail of user actions
- **invitations** - Team invitation system

### Astra-Specific Tables
- **projects** - User code projects
- **migrations** - Cross-language migration records
- **codebase_analytics** - Project health metrics

## API Routes

The following API endpoints are available:

### Dashboard Stats
```
GET /api/dashboard/stats
```
Returns user's dashboard statistics (projects, migrations, files processed)

### Projects
```
GET /api/projects
POST /api/projects
```
Manage user projects

### Migrations
```
GET /api/migrations?projectId=123
POST /api/migrations
```
Manage code migrations

## Supabase Dashboard Features

### Row Level Security (RLS)
Consider enabling RLS policies for additional security:

1. Go to **Authentication** → **Policies**
2. Enable RLS on tables
3. Create policies like:
   - Users can only read/write their own data
   - Team members can access team data

### Realtime (Optional)
Enable realtime subscriptions for live updates:

1. Go to **Database** → **Replication**
2. Enable replication for tables you want to subscribe to
3. Use Supabase client to subscribe to changes

### Storage (Optional)
If you need file uploads (e.g., project files):

1. Go to **Storage**
2. Create a bucket (e.g., "project-files")
3. Set up access policies
4. Use Supabase storage API in your app

## Troubleshooting

### Connection Issues
- Verify your `POSTGRES_URL` is correct
- Check that your IP is allowed (Supabase → Settings → Database → Connection pooling)
- Ensure password doesn't contain special characters that need URL encoding

### Migration Errors
- Delete `lib/db/migrations` folder and regenerate
- Check Supabase logs in dashboard
- Verify all environment variables are set

### Authentication Issues
- Verify `AUTH_SECRET` is set and consistent
- Check Supabase Auth settings (Settings → Authentication)
- Enable email provider in Supabase Auth

## Next Steps

1. **Enable Email Auth**: Configure email templates in Supabase
2. **Add OAuth**: Set up Google/GitHub OAuth in Supabase Auth
3. **Set up Stripe**: Add payment processing for premium features
4. **Deploy**: Deploy to Vercel and update environment variables
5. **Monitor**: Use Supabase dashboard to monitor usage and performance

## Resources

- [Supabase Documentation](https://supabase.com/docs)
- [Drizzle ORM Documentation](https://orm.drizzle.team/docs/overview)
- [Next.js Documentation](https://nextjs.org/docs)
