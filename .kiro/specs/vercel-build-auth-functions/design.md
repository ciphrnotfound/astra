# Vercel Build Auth Functions Bugfix Design

## Overview

The Vercel build is failing because multiple files across the application are importing authentication and team management functions that don't exist in `lib/db/queries.ts`. Additionally, the Button component is not properly exported from `components/ui/button.tsx`. This bug prevents the application from building and deploying, blocking all development and production deployments.

The fix involves implementing five missing database query functions (`getUser`, `getTeamForUser`, `getUserWithTeam`, `getTeamByStripeCustomerId`, `updateTeamSubscription`) and properly exporting the Button component. These implementations already exist in the template directory and need to be added to the main codebase.

## Glossary

- **Bug_Condition (C)**: The condition that triggers the bug - when the TypeScript compiler attempts to import functions that don't exist in the target module
- **Property (P)**: The desired behavior - all imports resolve successfully and the build completes without module resolution errors
- **Preservation**: Existing CLI dashboard query functions and database operations that must remain unchanged by the fix
- **getUser**: Function that retrieves the current authenticated user from the session cookie
- **getTeamForUser**: Function that retrieves the current user's team with all team members
- **getUserWithTeam**: Function that retrieves a user's team membership information by userId
- **getTeamByStripeCustomerId**: Function that queries for a team by Stripe customer ID
- **updateTeamSubscription**: Function that updates a team's subscription data in the database
- **Session Cookie**: HTTP-only cookie containing JWT token with user authentication data
- **TeamDataWithMembers**: Type representing a team with nested team member and user data

## Bug Details

### Bug Condition

The bug manifests when the TypeScript compiler attempts to build files that import authentication and team management functions from `@/lib/db/queries`. The compiler cannot resolve these imports because the functions are not defined or exported in the target module.

**Formal Specification:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type CompilationUnit (TypeScript file being compiled)
  OUTPUT: boolean
  
  RETURN input.imports CONTAINS importStatement
         WHERE importStatement.source == '@/lib/db/queries'
         AND importStatement.specifiers CONTAINS functionName
         WHERE functionName IN ['getUser', 'getTeamForUser', 'getUserWithTeam', 
                                'getTeamByStripeCustomerId', 'updateTeamSubscription']
         AND NOT functionExists(functionName, '@/lib/db/queries')
         
         OR (input.imports CONTAINS importStatement
             WHERE importStatement.source == '@/components/ui/button'
             AND importStatement.specifiers CONTAINS 'Button'
             AND NOT isExported('Button', '@/components/ui/button'))
END FUNCTION
```

### Examples

- `lib/payments/stripe.ts` imports `getUser`, `getTeamByStripeCustomerId`, and `updateTeamSubscription` from `@/lib/db/queries` → Build fails with "Module not found" error
- `lib/auth/middleware.ts` imports `getUser` and `getTeamForUser` from `@/lib/db/queries` → Build fails with "Module not found" error
- `app/(login)/actions.ts` imports `getUser` and `getUserWithTeam` from `@/lib/db/queries` → Build fails with "Module not found" error
- `app/(login)/login.tsx` imports `Button` from `@/components/ui/button` → Build fails because Button is exported as default instead of named export
- `app/api/user/route.ts` imports `getUser` from `@/lib/db/queries` → Build fails with "Module not found" error
- `app/api/team/route.ts` imports `getTeamForUser` from `@/lib/db/queries` → Build fails with "Module not found" error

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- All existing CLI dashboard query functions (`getDashboardStats`, `getHealthMetrics`, `getSecurityIssues`, etc.) must continue to work exactly as before
- Database operations for projects, migrations, and analytics must remain unchanged
- The Supabase client creation and configuration must remain unchanged
- All existing type definitions and imports must remain unchanged

**Scope:**
All code that does NOT involve authentication or team management queries should be completely unaffected by this fix. This includes:
- CLI dashboard statistics queries
- Health metrics and security issue queries
- Timeline events and task queries
- Dependency graph queries
- Astra session and migration queries
- All Supabase-based queries in the existing codebase

## Hypothesized Root Cause

Based on the bug description and code analysis, the root cause is clear:

1. **Incomplete Migration from Template**: The authentication and team management functions exist in the template directory (`template/saas-starter-main/lib/db/queries.ts`) but were never copied to the main codebase (`lib/db/queries.ts`)

2. **Missing Function Implementations**: Five critical functions are missing:
   - `getUser()` - retrieves authenticated user from session cookie
   - `getTeamForUser()` - retrieves user's team with members using Drizzle relational queries
   - `getUserWithTeam(userId)` - retrieves user's team membership by userId
   - `getTeamByStripeCustomerId(customerId)` - finds team by Stripe customer ID
   - `updateTeamSubscription(teamId, data)` - updates team subscription fields

3. **Incorrect Button Export**: The Button component uses `export default` instead of named export `export { Button }`, causing import failures in files expecting a named import

4. **Database Schema Mismatch**: The main queries file uses Supabase client while the template uses Drizzle ORM, requiring adaptation of the authentication functions to work with the existing database setup

## Correctness Properties

Property 1: Bug Condition - Module Resolution Success

_For any_ TypeScript file that imports authentication or team management functions from `@/lib/db/queries` or imports Button from `@/components/ui/button`, the TypeScript compiler SHALL successfully resolve all imports, and the build process SHALL complete without "Module not found" errors.

**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**

Property 2: Preservation - Existing Query Functions

_For any_ existing CLI dashboard query function (getDashboardStats, getHealthMetrics, getSecurityIssues, etc.), the fixed code SHALL produce exactly the same results and behavior as the original code, preserving all existing database query functionality.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4**

## Fix Implementation

### Changes Required

**File 1**: `astra-landing-next/lib/db/queries.ts`

**Function Additions**:

1. **Add getUser() function**:
   - Retrieve session cookie using Next.js cookies API
   - Verify JWT token using verifyToken from auth/session module
   - Check token expiration
   - Query users table using Drizzle ORM with userId from session
   - Filter out soft-deleted users (deletedAt IS NULL)
   - Return User object or null if not authenticated

2. **Add getTeamForUser() function**:
   - Call getUser() to get authenticated user
   - Return null if user not authenticated
   - Use Drizzle relational query on teamMembers table
   - Join with teams table and nested teamMembers with users
   - Select only id, name, email from nested user objects
   - Return TeamDataWithMembers or null

3. **Add getUserWithTeam(userId: number) function**:
   - Query users table joined with teamMembers table
   - Use left join to handle users without teams
   - Select user object and teamId
   - Return object with user and teamId properties

4. **Add getTeamByStripeCustomerId(customerId: string) function**:
   - Query teams table with stripeCustomerId filter
   - Use eq() condition from Drizzle
   - Limit to 1 result
   - Return Team object or null

5. **Add updateTeamSubscription(teamId, subscriptionData) function**:
   - Update teams table with subscription data
   - Set stripeSubscriptionId, stripeProductId, planName, subscriptionStatus
   - Update updatedAt timestamp to current date
   - Use eq() condition to filter by teamId

**Required Imports**:
- Add: `import { verifyToken } from '@/lib/auth/session';`
- Add: `import { cookies } from 'next/headers';`
- Ensure Drizzle imports include: `desc, and, eq, isNull`
- Ensure schema imports include: `activityLogs, teamMembers, teams, users`

**File 2**: `astra-landing-next/components/ui/button.tsx`

**Export Changes**:
- Change from `export default function Button(props: ButtonProps)` to named export
- Add proper TypeScript types for ButtonProps
- Export both Button component and buttonVariants (if using class-variance-authority pattern)
- Ensure compatibility with existing usage patterns (loading, variant, size props)

## Testing Strategy

### Validation Approach

The testing strategy follows a two-phase approach: first, verify the build fails on unfixed code to confirm the bug, then verify the build succeeds after implementing the fix and that all functionality works correctly.

### Exploratory Bug Condition Checking

**Goal**: Confirm the build failure on unfixed code and identify all files affected by missing imports.

**Test Plan**: Attempt to build the application without the fixes and capture all TypeScript compilation errors related to module resolution.

**Test Cases**:
1. **Build Failure Test**: Run `npm run build` or `vercel build` on unfixed code (will fail with multiple "Module not found" errors)
2. **Import Resolution Test**: Check TypeScript language server errors in IDE for all affected files (will show red squiggles on imports)
3. **Function Usage Test**: Verify that calling code expects specific function signatures (will help validate implementation correctness)

**Expected Counterexamples**:
- TypeScript error: "Module '@/lib/db/queries' has no exported member 'getUser'"
- TypeScript error: "Module '@/lib/db/queries' has no exported member 'getTeamForUser'"
- TypeScript error: "Module '@/lib/db/queries' has no exported member 'getUserWithTeam'"
- TypeScript error: "Module '@/lib/db/queries' has no exported member 'getTeamByStripeCustomerId'"
- TypeScript error: "Module '@/lib/db/queries' has no exported member 'updateTeamSubscription'"
- TypeScript error: "Module '@/components/ui/button' has no exported member 'Button'"

### Fix Checking

**Goal**: Verify that after implementing the missing functions and fixing the Button export, the build completes successfully.

**Pseudocode:**
```
FOR ALL file WHERE isBugCondition(file) DO
  result := compileTypeScript(file)
  ASSERT result.success == true
  ASSERT result.errors.length == 0
END FOR

ASSERT buildApplication().success == true
```

**Test Plan**:
1. Implement all five missing functions in `lib/db/queries.ts`
2. Fix Button component export in `components/ui/button.tsx`
3. Run TypeScript compiler to check for errors
4. Run full build process (Vercel build or npm run build)
5. Verify no module resolution errors occur

### Preservation Checking

**Goal**: Verify that all existing CLI dashboard functions continue to work correctly after adding the new authentication functions.

**Pseudocode:**
```
FOR ALL existingFunction IN ['getDashboardStats', 'getHealthMetrics', 
                              'getSecurityIssues', 'updateSecurityIssueStatus',
                              'getTimelineEvents', 'getCLITasks', 
                              'updateCLITaskStatus', 'getDependencyGraph',
                              'getAstraSessions', 'getMigrations'] DO
  ASSERT existingFunction_original(input) == existingFunction_fixed(input)
END FOR
```

**Testing Approach**: Since we're only adding new functions and not modifying existing ones, preservation is guaranteed by the nature of the change. However, we should verify:
- No import conflicts or naming collisions
- No changes to existing function signatures
- No modifications to existing database queries

**Test Plan**: 
1. **Code Review**: Verify that no existing functions are modified in the diff
2. **Import Test**: Ensure new imports don't conflict with existing imports
3. **Type Check**: Run TypeScript compiler to ensure no type errors in existing code
4. **Runtime Test**: If possible, test one existing function (e.g., getDashboardStats) to ensure it still works

**Test Cases**:
1. **No Modification Test**: Verify git diff shows only additions, no modifications to existing functions
2. **Import Compatibility Test**: Verify all existing imports still resolve correctly
3. **Type Safety Test**: Run `tsc --noEmit` to check for type errors
4. **Button Props Test**: Verify Button component still accepts all existing props (loading, variant, size, etc.)

### Unit Tests

- Test getUser() returns null when no session cookie exists
- Test getUser() returns null when session token is expired
- Test getUser() returns User object when valid session exists
- Test getTeamForUser() returns null when user not authenticated
- Test getTeamForUser() returns TeamDataWithMembers when user has team
- Test getUserWithTeam() returns user with teamId when user is team member
- Test getUserWithTeam() returns user with null teamId when user has no team
- Test getTeamByStripeCustomerId() returns Team when customer ID matches
- Test getTeamByStripeCustomerId() returns null when no match found
- Test updateTeamSubscription() updates all subscription fields correctly
- Test Button component renders with all variant types
- Test Button component shows loading spinner when loading prop is true

### Property-Based Tests

- Generate random session tokens (valid and invalid) and verify getUser() handles them correctly
- Generate random user IDs and verify getUserWithTeam() returns consistent results
- Generate random Stripe customer IDs and verify getTeamByStripeCustomerId() handles edge cases
- Generate random subscription data and verify updateTeamSubscription() updates correctly
- Generate random Button props combinations and verify component renders without errors

### Integration Tests

- Test full authentication flow: sign in → getUser() → verify user data
- Test team management flow: create team → getTeamForUser() → verify team data
- Test Stripe webhook flow: receive webhook → getTeamByStripeCustomerId() → updateTeamSubscription()
- Test API routes: call /api/user → verify getUser() is called correctly
- Test API routes: call /api/team → verify getTeamForUser() is called correctly
- Test login page: verify Button component renders and handles clicks
- Test full Vercel build: verify application builds and deploys successfully
