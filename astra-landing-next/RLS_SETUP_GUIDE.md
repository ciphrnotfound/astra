# Row Level Security (RLS) Setup Guide

This guide explains how to apply and test the Row Level Security policies for the Astra application.

## Overview

Row Level Security (RLS) ensures that users can only access their own data. When RLS is enabled, PostgreSQL automatically filters query results based on the authenticated user's context, preventing unauthorized data access.

## What's Included

### 1. RLS Migration File (`supabase-migration-002-rls.sql`)

This file contains:
- RLS enablement for all 12 tables
- Comprehensive security policies for each table
- User data isolation rules

### 2. Test Script (`test-rls-policies.sql`)

This file contains:
- Test data setup (2 test users with projects, migrations, etc.)
- 12 automated test cases verifying policy enforcement
- Cleanup scripts (optional)

## Tables Protected by RLS

1. **users** - Users can only view/update their own profile
2. **projects** - Users can only access their own projects
3. **migrations** - Users can only access their own migrations
4. **codebase_analytics** - Users can access analytics for their projects
5. **api_keys** - Users can only access their own API keys
6. **vulnerabilities** - Users can access vulnerabilities in their projects
7. **tasks** - Users can access tasks they created, are assigned to, or belong to their projects
8. **timeline_events** - Users can access timeline events for their projects
9. **learning_phases** - All authenticated users can read (public content)
10. **user_progress** - Users can only access their own learning progress
11. **dependencies** - Users can access dependencies for their projects
12. **user_settings** - Users can only access their own settings

## How to Apply RLS Policies

### Step 1: Access Supabase SQL Editor

1. Go to your Supabase project dashboard
2. Navigate to the **SQL Editor** section
3. Click **New Query**

### Step 2: Run the RLS Migration

1. Open the file `supabase-migration-002-rls.sql`
2. Copy the entire contents
3. Paste into the Supabase SQL Editor
4. Click **Run** or press `Ctrl+Enter` (Windows/Linux) or `Cmd+Enter` (Mac)

You should see a success message indicating all policies were created.

### Step 3: Verify RLS is Enabled

Run this query to check RLS status:

```sql
SELECT 
  schemaname,
  tablename,
  rowsecurity
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY tablename;
```

All tables should show `rowsecurity = true`.

## How to Test RLS Policies

### Option 1: Automated Test Script (Recommended)

1. Open the file `test-rls-policies.sql`
2. Copy the entire contents
3. Paste into the Supabase SQL Editor
4. Click **Run**

The script will:
- Create test users (Alice and Bob)
- Create test data for all tables
- Run 12 automated test cases
- Display PASS/FAIL results for each test
- Show a summary of all tested tables

**Expected Output:**
```
NOTICE: PASS: Users table RLS working correctly
NOTICE: PASS: Projects table RLS working correctly
NOTICE: PASS: Migrations table RLS working correctly
...
NOTICE: All 12 test cases passed successfully!
```

### Option 2: Manual Testing

You can manually test policies by simulating different user contexts:

```sql
-- Simulate user with ID 1
SELECT set_config('request.jwt.claims', '{"sub": "1"}', true);

-- Try to query projects (should only see user 1's projects)
SELECT * FROM projects;

-- Simulate user with ID 2
SELECT set_config('request.jwt.claims', '{"sub": "2"}', true);

-- Try to query projects (should only see user 2's projects)
SELECT * FROM projects;
```

## Understanding the Policies

### Direct Ownership Policies

Tables where users directly own records:
- `users.id = auth.uid()`
- `projects.user_id = auth.uid()`
- `migrations.user_id = auth.uid()`
- `api_keys.user_id = auth.uid()`
- `user_settings.user_id = auth.uid()`
- `user_progress.user_id = auth.uid()`

### Project-Based Policies

Tables where access is granted through project ownership:
- `vulnerabilities` - Access if you own the project
- `timeline_events` - Access if you own the project
- `dependencies` - Access if you own the project
- `codebase_analytics` - Access if you own the project

### Multi-Condition Policies

**Tasks Table:**
Users can access tasks if:
- They created the task (`created_by = auth.uid()`)
- OR they are assigned to the task (`assignee_id = auth.uid()`)
- OR the task belongs to their project

**Learning Phases:**
- All authenticated users can read (public educational content)
- Write access should be restricted to admins (to be implemented)

## Security Best Practices

### 1. Always Use Authenticated Queries

When querying from your application, always ensure the user is authenticated:

```typescript
// Good - Uses authenticated Supabase client
const supabase = createServerClient();
const { data } = await supabase.from('projects').select('*');
// RLS automatically filters to user's projects

// Bad - Using service role bypasses RLS
const supabase = createClient(url, SERVICE_ROLE_KEY);
// This bypasses RLS - only use for admin operations!
```

### 2. Test with Multiple Users

Always test your application with multiple user accounts to ensure data isolation works correctly.

### 3. Monitor Policy Performance

Complex policies with subqueries can impact performance. Monitor query execution times and add indexes as needed.

### 4. Audit Policy Changes

Keep track of all policy changes in migration files for version control and rollback capability.

## Troubleshooting

### Issue: Policies Not Working

**Symptom:** Users can see data they shouldn't access

**Solutions:**
1. Verify RLS is enabled: `SELECT rowsecurity FROM pg_tables WHERE tablename = 'your_table';`
2. Check if you're using the service role key (bypasses RLS)
3. Verify `auth.uid()` returns the correct user ID
4. Check policy conditions match your data structure

### Issue: Users Can't See Their Own Data

**Symptom:** Queries return empty results even for owned data

**Solutions:**
1. Verify the user is authenticated: `SELECT auth.uid();`
2. Check foreign key relationships (e.g., `project_id` matches an owned project)
3. Verify data types match (e.g., `auth.uid()::text` vs `user_id::text`)
4. Check for NULL values in join conditions

### Issue: Performance Problems

**Symptom:** Queries are slow after enabling RLS

**Solutions:**
1. Add indexes on columns used in policy conditions
2. Simplify complex policy subqueries
3. Use `EXPLAIN ANALYZE` to identify bottlenecks
4. Consider denormalizing data to reduce joins

## Next Steps

After successfully applying and testing RLS policies:

1. ✅ **Task 1.2 Complete** - RLS policies implemented and tested
2. ⏭️ **Task 1.3** - Update TypeScript types in `lib/db/types.ts`
3. ⏭️ **Task 1.4** - Create database query utilities in `lib/db/queries.ts`

## Additional Resources

- [Supabase RLS Documentation](https://supabase.com/docs/guides/auth/row-level-security)
- [PostgreSQL RLS Documentation](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
- [Supabase Auth Helpers](https://supabase.com/docs/guides/auth/auth-helpers)

## Support

If you encounter issues:
1. Check the Supabase logs in your dashboard
2. Review the test script output for specific failures
3. Verify your authentication setup is working correctly
4. Consult the Supabase community or documentation
