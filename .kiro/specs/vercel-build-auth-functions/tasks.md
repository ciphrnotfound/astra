# Implementation Plan

- [ ] 1. Write bug condition exploration test
  - **Property 1: Bug Condition** - Module Resolution Failure
  - **CRITICAL**: This test MUST FAIL on unfixed code - failure confirms the bug exists
  - **DO NOT attempt to fix the test or the code when it fails**
  - **NOTE**: This test encodes the expected behavior - it will validate the fix when it passes after implementation
  - **GOAL**: Surface counterexamples that demonstrate the bug exists
  - **Scoped PBT Approach**: Scope the property to concrete failing cases - files importing missing functions from '@/lib/db/queries' and Button from '@/components/ui/button'
  - Test that TypeScript compilation fails for files importing 'getUser', 'getTeamForUser', 'getUserWithTeam', 'getTeamByStripeCustomerId', 'updateTeamSubscription' from '@/lib/db/queries'
  - Test that TypeScript compilation fails for files importing 'Button' from '@/components/ui/button'
  - Run test on UNFIXED code
  - **EXPECTED OUTCOME**: Test FAILS (this is correct - it proves the bug exists)
  - Document counterexamples found: "Module '@/lib/db/queries' has no exported member 'getUser'", etc.
  - Mark task complete when test is written, run, and failure is documented
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [ ] 2. Write preservation property tests (BEFORE implementing fix)
  - **Property 2: Preservation** - Existing Query Functions
  - **IMPORTANT**: Follow observation-first methodology
  - Observe behavior on UNFIXED code for existing CLI dashboard functions (getDashboardStats, getHealthMetrics, getSecurityIssues, etc.)
  - Write property-based tests capturing that existing functions remain unchanged and produce same results
  - Property-based testing generates many test cases for stronger guarantees
  - Run tests on UNFIXED code
  - **EXPECTED OUTCOME**: Tests PASS (this confirms baseline behavior to preserve)
  - Mark task complete when tests are written, run, and passing on unfixed code
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 3. Fix for Vercel build failure by implementing missing authentication and team management functions

  - [x] 3.1 Implement getUser() function in lib/db/queries.ts
    - Retrieve session cookie using Next.js cookies API
    - Verify JWT token using verifyToken from auth/session module
    - Check token expiration
    - Query users table using Drizzle ORM with userId from session
    - Filter out soft-deleted users (deletedAt IS NULL)
    - Return User object or null if not authenticated
    - Add required imports: verifyToken from '@/lib/auth/session', cookies from 'next/headers'
    - _Bug_Condition: isBugCondition(input) where input.imports contains 'getUser' from '@/lib/db/queries' AND NOT functionExists('getUser', '@/lib/db/queries')_
    - _Expected_Behavior: TypeScript compiler successfully resolves getUser import and build completes without errors_
    - _Preservation: Existing CLI dashboard query functions remain unchanged_
    - _Requirements: 2.1, 3.1, 3.2_

  - [x] 3.2 Implement getTeamForUser() function in lib/db/queries.ts
    - Call getUser() to get authenticated user
    - Return null if user not authenticated
    - Use Drizzle relational query on teamMembers table
    - Join with teams table and nested teamMembers with users
    - Select only id, name, email from nested user objects
    - Return TeamDataWithMembers or null
    - Ensure schema imports include teamMembers, teams, users
    - _Bug_Condition: isBugCondition(input) where input.imports contains 'getTeamForUser' from '@/lib/db/queries' AND NOT functionExists('getTeamForUser', '@/lib/db/queries')_
    - _Expected_Behavior: TypeScript compiler successfully resolves getTeamForUser import and build completes without errors_
    - _Preservation: Existing CLI dashboard query functions remain unchanged_
    - _Requirements: 2.2, 3.1, 3.2_

  - [x] 3.3 Implement getUserWithTeam() function in lib/db/queries.ts
    - Query users table joined with teamMembers table
    - Use left join to handle users without teams
    - Select user object and teamId
    - Return object with user and teamId properties
    - _Bug_Condition: isBugCondition(input) where input.imports contains 'getUserWithTeam' from '@/lib/db/queries' AND NOT functionExists('getUserWithTeam', '@/lib/db/queries')_
    - _Expected_Behavior: TypeScript compiler successfully resolves getUserWithTeam import and build completes without errors_
    - _Preservation: Existing CLI dashboard query functions remain unchanged_
    - _Requirements: 2.3, 3.1, 3.2_

  - [x] 3.4 Implement getTeamByStripeCustomerId() function in lib/db/queries.ts
    - Query teams table with stripeCustomerId filter
    - Use eq() condition from Drizzle
    - Limit to 1 result
    - Return Team object or null
    - Ensure Drizzle imports include eq
    - _Bug_Condition: isBugCondition(input) where input.imports contains 'getTeamByStripeCustomerId' from '@/lib/db/queries' AND NOT functionExists('getTeamByStripeCustomerId', '@/lib/db/queries')_
    - _Expected_Behavior: TypeScript compiler successfully resolves getTeamByStripeCustomerId import and build completes without errors_
    - _Preservation: Existing CLI dashboard query functions remain unchanged_
    - _Requirements: 2.4, 3.1, 3.2_

  - [x] 3.5 Implement updateTeamSubscription() function in lib/db/queries.ts
    - Update teams table with subscription data
    - Set stripeSubscriptionId, stripeProductId, planName, subscriptionStatus
    - Update updatedAt timestamp to current date
    - Use eq() condition to filter by teamId
    - _Bug_Condition: isBugCondition(input) where input.imports contains 'updateTeamSubscription' from '@/lib/db/queries' AND NOT functionExists('updateTeamSubscription', '@/lib/db/queries')_
    - _Expected_Behavior: TypeScript compiler successfully resolves updateTeamSubscription import and build completes without errors_
    - _Preservation: Existing CLI dashboard query functions remain unchanged_
    - _Requirements: 2.5, 3.1, 3.2_

  - [x] 3.6 Fix Button component export in components/ui/button.tsx
    - Change from export default to named export
    - Export Button component and buttonVariants
    - Ensure compatibility with existing usage patterns (loading, variant, size props)
    - _Bug_Condition: isBugCondition(input) where input.imports contains 'Button' from '@/components/ui/button' AND NOT isExported('Button', '@/components/ui/button')_
    - _Expected_Behavior: TypeScript compiler successfully resolves Button import and build completes without errors_
    - _Preservation: Button component still accepts all existing props_
    - _Requirements: 2.6, 3.3, 3.4_

  - [ ] 3.7 Verify bug condition exploration test now passes
    - **Property 1: Expected Behavior** - Module Resolution Success
    - **IMPORTANT**: Re-run the SAME test from task 1 - do NOT write a new test
    - The test from task 1 encodes the expected behavior
    - When this test passes, it confirms the expected behavior is satisfied
    - Run bug condition exploration test from step 1
    - **EXPECTED OUTCOME**: Test PASSES (confirms bug is fixed)
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [ ] 3.8 Verify preservation tests still pass
    - **Property 2: Preservation** - Existing Query Functions
    - **IMPORTANT**: Re-run the SAME tests from task 2 - do NOT write new tests
    - Run preservation property tests from step 2
    - **EXPECTED OUTCOME**: Tests PASS (confirms no regressions)
    - Confirm all tests still pass after fix (no regressions)
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 4. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
