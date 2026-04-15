# Bugfix Requirements Document

## Introduction

The Vercel build is failing due to missing authentication and team management functions in the database queries module. Multiple files across the application (authentication middleware, login actions, API routes, and Stripe payment integration) are importing functions that don't exist in `lib/db/queries.ts`. Additionally, the Button component is not being exported from `components/ui/button.tsx`, causing import failures in the login component.

This bug prevents the application from building and deploying to Vercel, blocking all development and production deployments.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN the Vercel build process attempts to compile `lib/payments/stripe.ts` THEN the build fails with "Module not found" errors for `getUser`, `getTeamByStripeCustomerId`, and `updateTeamSubscription` imports from `@/lib/db/queries`

1.2 WHEN the Vercel build process attempts to compile `lib/auth/middleware.ts` THEN the build fails with "Module not found" errors for `getUser` and `getTeamForUser` imports from `@/lib/db/queries`

1.3 WHEN the Vercel build process attempts to compile `app/(login)/actions.ts` THEN the build fails with "Module not found" errors for `getUser` and `getUserWithTeam` imports from `@/lib/db/queries`

1.4 WHEN the Vercel build process attempts to compile `app/(login)/login.tsx` THEN the build fails with "Module not found" error for `Button` import from `@/components/ui/button`

1.5 WHEN the Vercel build process attempts to compile `app/api/user/route.ts` THEN the build fails with "Module not found" error for `getUser` import from `@/lib/db/queries`

1.6 WHEN the Vercel build process attempts to compile `app/api/team/route.ts` THEN the build fails with "Module not found" error for `getTeamForUser` import from `@/lib/db/queries`

### Expected Behavior (Correct)

2.1 WHEN the Vercel build process attempts to compile `lib/payments/stripe.ts` THEN the system SHALL successfully import and use `getUser`, `getTeamByStripeCustomerId`, and `updateTeamSubscription` functions from `@/lib/db/queries`

2.2 WHEN the Vercel build process attempts to compile `lib/auth/middleware.ts` THEN the system SHALL successfully import and use `getUser` and `getTeamForUser` functions from `@/lib/db/queries`

2.3 WHEN the Vercel build process attempts to compile `app/(login)/actions.ts` THEN the system SHALL successfully import and use `getUser` and `getUserWithTeam` functions from `@/lib/db/queries`

2.4 WHEN the Vercel build process attempts to compile `app/(login)/login.tsx` THEN the system SHALL successfully import and use the `Button` component from `@/components/ui/button`

2.5 WHEN the Vercel build process attempts to compile `app/api/user/route.ts` THEN the system SHALL successfully import and use `getUser` function from `@/lib/db/queries`

2.6 WHEN the Vercel build process attempts to compile `app/api/team/route.ts` THEN the system SHALL successfully import and use `getTeamForUser` function from `@/lib/db/queries`

2.7 WHEN `getUser()` is called THEN the system SHALL retrieve the current user from the session cookie, query the database for the full user record, and return the User object or null if not authenticated

2.8 WHEN `getTeamForUser()` is called THEN the system SHALL retrieve the current user's team with all team members and their details, returning a TeamDataWithMembers object or null if the user has no team

2.9 WHEN `getUserWithTeam(userId)` is called THEN the system SHALL retrieve the user's team membership information including the teamId and role, returning the team member record or null if not found

2.10 WHEN `getTeamByStripeCustomerId(customerId)` is called THEN the system SHALL query the database for a team with the matching Stripe customer ID and return the Team object or null if not found

2.11 WHEN `updateTeamSubscription(teamId, data)` is called THEN the system SHALL update the team's subscription fields (stripeSubscriptionId, stripeProductId, planName, subscriptionStatus) in the database

### Unchanged Behavior (Regression Prevention)

3.1 WHEN existing CLI dashboard functions in `lib/db/queries.ts` are called THEN the system SHALL CONTINUE TO function correctly without any changes to their behavior

3.2 WHEN the application performs database operations for projects, migrations, and analytics THEN the system SHALL CONTINUE TO work as expected without modification

3.3 WHEN users interact with non-authentication features of the application THEN the system SHALL CONTINUE TO function normally

3.4 WHEN the Button component is used with its existing props (loading, variant, size, etc.) THEN the system SHALL CONTINUE TO render and behave identically
