# Implementation Plan: Content and Backend Integration

## Overview

This plan implements the complete Astra landing page and dashboard with real content and full backend integration. The implementation follows a phased approach: database schema extensions, authentication and data infrastructure, dashboard pages with real data, documentation system, footer pages, and final polish.

## Tasks

- [x] 1. Extend database schema and setup backend infrastructure
  - [x] 1.1 Create database migration file with all new tables
    - Add vulnerabilities, tasks, timeline_events, learning_phases, user_progress, dependencies, user_settings tables
    - Include all indexes and constraints from design
    - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5, 18.6, 18.7, 18.8, 18.9, 18.10_
  
  - [x] 1.2 Implement Row Level Security (RLS) policies
    - Create RLS policies for all tables ensuring user data isolation
    - Test policies with different user contexts
    - _Requirements: 13.5, 18.9_
  
  - [x] 1.3 Create TypeScript types for all database tables
    - Update lib/db/types.ts with interfaces for new tables
    - Export all types for use across application
    - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5, 18.6, 18.7_
  
  - [x] 1.4 Create database query utilities
    - Implement reusable query functions in lib/db/queries.ts
    - Include functions for dashboard stats, health metrics, migrations, vulnerabilities, tasks, timeline events
    - _Requirements: 19.1, 19.2, 19.3, 19.4, 19.5, 19.6, 19.7, 19.8, 19.9, 19.10_

- [ ] 2. Implement authentication and session management
  - [x] 2.1 Create authentication middleware
    - Implement requireAuth function in lib/supabase/middleware.ts
    - Handle session validation and redirects
    - _Requirements: 13.1, 13.4_
  
  - [x] 2.2 Update dashboard layout with auth protection
    - Add authentication check to app/dashboard/layout.tsx
    - Display authenticated user information in navigation
    - _Requirements: 13.2, 13.7, 13.8_
  
  - [x] 2.3 Implement sign-out functionality
    - Add sign-out handler to user menu
    - Clear session and redirect to landing page
    - _Requirements: 13.6_
  
  - [x] 2.4 Add session refresh handling
    - Implement automatic session refresh
    - Handle expired sessions gracefully
    - _Requirements: 13.3, 13.10_

- [ ] 3. Create shared UI components and hooks
  - [x] 3.1 Create dashboard UI components
    - Implement StatCard, EmptyState, LoadingState, ErrorBoundary components
    - Add DataTable with pagination, sorting, filtering
    - _Requirements: 14.1, 14.2, 14.3, 15.1, 15.5_
  
  - [x] 3.2 Create data fetching hooks
    - Implement useDashboardStats, useHealthMetrics, useRealtime hooks
    - Include loading, error, and refetch states
    - _Requirements: 3.8, 14.1, 14.2, 14.3, 14.4_
  
  - [x] 3.3 Create form components
    - Implement ProjectForm, MigrationForm, TaskForm, SettingsForm
    - Add validation and error handling
    - _Requirements: 14.6, 14.7, 14.8_
  
  - [-] 3.4 Create chart components
    - Implement MetricChart for line/bar charts
    - Support multiple time ranges
    - _Requirements: 4.8, 4.10_

- [ ] 4. Implement dashboard overview page with real data
  - [ ] 4.1 Create dashboard stats API endpoint
    - Implement GET /api/dashboard/stats route
    - Fetch total migrations, files processed, active projects
    - _Requirements: 3.1, 3.2, 3.3, 19.1_
  
  - [ ] 4.2 Update dashboard overview page
    - Fetch and display real statistics from Supabase
    - Show 5 most recent migrations
    - Display projects with last updated timestamps
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [ ] 4.3 Add empty states for dashboard
    - Display empty state when no migrations exist
    - Display empty state when no projects exist
    - Include call-to-action buttons
    - _Requirements: 3.6, 3.7_
  
  - [ ] 4.4 Add error handling and retry logic
    - Display error messages on fetch failures
    - Provide retry button
    - _Requirements: 3.9, 14.3, 14.4, 14.5_
  
  - [ ]* 4.5 Implement real-time updates for dashboard
    - Subscribe to migration and project changes
    - Update display without page refresh
    - _Requirements: 3.10, 16.1, 16.2_

- [ ] 5. Implement health metrics page
  - [ ] 5.1 Create health metrics API endpoint
    - Implement GET /api/health route with time range filtering
    - Fetch all six health metrics and historical trends
    - _Requirements: 4.1, 19.4_
  
  - [ ] 5.2 Create health metrics display components
    - Implement metric cards with visual indicators
    - Add trend indicators (up/down arrows)
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_
  
  - [ ] 5.3 Implement trend visualization
    - Create line charts showing 30-day trends
    - Support time range selection (7d, 30d, 90d)
    - _Requirements: 4.8, 4.10_
  
  - [ ] 5.4 Add insufficient data handling
    - Display message when historical data is limited
    - _Requirements: 4.9_
  
  - [ ]* 5.5 Implement real-time health metric updates
    - Subscribe to codebase_analytics changes
    - Update metrics without page refresh
    - _Requirements: 16.4_

- [ ] 6. Checkpoint - Verify dashboard core functionality
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Implement dependency graph visualization (Semantic Cartographer)
  - [ ] 7.1 Create dependencies API endpoint
    - Implement GET /api/graph route
    - Fetch dependency data for selected project
    - _Requirements: 5.1, 19.5_
  
  - [ ] 7.2 Set up graph visualization library
    - Install and configure React Flow or D3.js
    - Create base graph component structure
    - _Requirements: 5.2_
  
  - [ ] 7.3 Implement interactive graph features
    - Add node click to highlight connected dependencies
    - Add hover tooltips with file metadata
    - Implement zoom and pan controls
    - _Requirements: 5.3, 5.4, 5.5_
  
  - [ ] 7.4 Add graph filtering and search
    - Implement search to locate specific files
    - Add filters for dependency types
    - Color-code nodes by file type
    - _Requirements: 5.6, 5.7, 5.8_
  
  - [ ] 7.5 Add empty state and performance optimization
    - Display empty state when no dependencies exist
    - Optimize rendering for large graphs (up to 10,000 files)
    - _Requirements: 5.9, 5.10_

- [ ] 8. Implement security vulnerability tracking (Security Hunter)
  - [ ] 8.1 Create vulnerabilities API endpoint
    - Implement GET /api/security route with filtering
    - Support severity, status, and file type filters
    - _Requirements: 7.1, 19.6_
  
  - [ ] 8.2 Create vulnerability display components
    - Group vulnerabilities by severity
    - Display counts for each severity level
    - Show affected files and descriptions
    - _Requirements: 7.2, 7.3, 7.4, 7.5_
  
  - [ ] 8.3 Implement vulnerability management
    - Add mark as resolved functionality
    - Update status in Supabase
    - _Requirements: 7.7_
  
  - [ ] 8.4 Add security score trend and export
    - Display security score trend over time
    - Implement export functionality for reports
    - _Requirements: 7.8, 7.10_
  
  - [ ] 8.5 Add empty state for clean codebases
    - Display success message when no vulnerabilities exist
    - _Requirements: 7.9_

- [ ] 9. Implement task management system
  - [ ] 9.1 Create tasks API endpoints
    - Implement GET /api/tasks route with filtering
    - Implement POST /api/tasks route for task creation
    - Implement PATCH /api/tasks/[id] route for updates
    - _Requirements: 8.1, 8.4, 8.6, 19.7, 19.8_
  
  - [ ] 9.2 Create task board UI
    - Display tasks grouped by status (todo, in_progress, done)
    - Implement drag-and-drop for status updates
    - _Requirements: 8.2, 8.5_
  
  - [ ] 9.3 Implement task creation and editing
    - Create task form with title, description, assignee fields
    - Add validation and error handling
    - _Requirements: 8.3, 8.4_
  
  - [ ] 9.4 Add team velocity metrics
    - Display tasks completed per week
    - Show task assignment distribution
    - _Requirements: 8.7, 8.8_
  
  - [ ] 9.5 Add filtering functionality
    - Implement filters for assignee, priority, tags
    - _Requirements: 8.9_
  
  - [ ]* 9.6 Implement real-time task updates
    - Subscribe to task changes
    - Update board without page refresh
    - _Requirements: 8.10, 16.3_

- [ ] 10. Implement codebase memory timeline
  - [ ] 10.1 Create timeline events API endpoint
    - Implement GET /api/timeline route with pagination
    - Support filtering by event type and date range
    - _Requirements: 9.1, 19.9_
  
  - [ ] 10.2 Create timeline display components
    - Display events in reverse chronological order
    - Show event types, timestamps, affected files
    - _Requirements: 9.2, 9.3, 9.4, 9.5_
  
  - [ ] 10.3 Implement timeline filtering and search
    - Add filters for event type and date range
    - Implement search functionality
    - _Requirements: 9.6, 9.8_
  
  - [ ] 10.4 Add event detail view
    - Display detailed information on event click
    - _Requirements: 9.7_
  
  - [ ] 10.5 Create visual timeline with pagination
    - Render visual timeline with date markers
    - Implement pagination for large histories
    - _Requirements: 9.9, 9.10_

- [ ] 11. Implement onboarding learning system
  - [ ] 11.1 Create learning phases API endpoint
    - Implement GET /api/onboarding route
    - Fetch phases and user progress
    - _Requirements: 10.1, 19.10_
  
  - [ ] 11.2 Create learning phase display
    - Display phases in recommended order
    - Show completion status and estimated time
    - _Requirements: 10.2, 10.3, 10.4_
  
  - [ ] 11.3 Implement phase content viewer
    - Display phase content and exercises on start
    - Render code examples with syntax highlighting
    - _Requirements: 10.5, 10.8_
  
  - [ ] 11.4 Add progress tracking
    - Update completion status in Supabase
    - Display overall progress percentage
    - Show badges for completed milestones
    - _Requirements: 10.6, 10.7, 10.10_
  
  - [ ] 11.5 Implement phase recommendations
    - Recommend next phases based on completion history
    - _Requirements: 10.9_

- [ ] 12. Checkpoint - Verify all dashboard pages functional
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 13. Implement settings management
  - [ ] 13.1 Create user settings API endpoints
    - Implement GET /api/settings route
    - Implement PATCH /api/settings route
    - _Requirements: 11.1, 11.3, 11.5, 11.9_
  
  - [ ] 13.2 Create persona configuration UI
    - Add form for developer role, experience level, preferences
    - Save changes to Supabase
    - _Requirements: 11.2, 11.3_
  
  - [ ] 13.3 Create model configuration UI
    - Add form for AI model selection and parameters
    - Save changes to Supabase
    - _Requirements: 11.4, 11.5_
  
  - [ ] 13.4 Create team management UI
    - Display team members with roles
    - Add invite member functionality with email
    - Add remove member functionality
    - _Requirements: 11.6, 11.7_
  
  - [ ] 13.5 Create integration configuration UI
    - Add forms for IDE plugins, CI/CD webhooks, Git connections
    - Validate credentials before saving
    - _Requirements: 11.8, 11.9_
  
  - [ ] 13.6 Add success/error messaging
    - Display feedback for all configuration changes
    - _Requirements: 11.10_

- [ ] 14. Implement project management
  - [ ] 14.1 Create projects API endpoints
    - Implement POST /api/projects route for creation
    - Implement PATCH /api/projects/[id] route for updates
    - Implement DELETE /api/projects/[id] route for deletion
    - _Requirements: 12.4, 12.8, 12.10_
  
  - [ ] 14.2 Create project creation form
    - Add form with name, description, repository URL fields
    - Validate required fields
    - _Requirements: 12.1, 12.2, 12.3_
  
  - [ ] 14.3 Implement project creation flow
    - Create project record in Supabase
    - Redirect to project detail page
    - _Requirements: 12.4, 12.5_
  
  - [ ] 14.4 Create project detail page
    - Display project metadata and associated migrations
    - Add edit functionality
    - Add delete functionality with confirmation
    - _Requirements: 12.6, 12.7, 12.9, 12.10_

- [ ] 15. Implement migrations management
  - [ ] 15.1 Create migrations API endpoints
    - Implement GET /api/migrations route with pagination
    - Implement POST /api/migrations route for creation
    - _Requirements: 6.1, 6.10, 19.2, 19.3_
  
  - [ ] 15.2 Create migrations list page
    - Display migrations in reverse chronological order
    - Show status, languages, files processed
    - Display error messages for failed migrations
    - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.6_
  
  - [ ] 15.3 Create migration detail view
    - Display detailed migration report on click
    - _Requirements: 6.7_
  
  - [ ] 15.4 Create new migration form
    - Add configuration form for new migrations
    - Submit to create migration record
    - _Requirements: 6.8, 6.9, 6.10_
  
  - [ ]* 15.5 Implement real-time migration status updates
    - Subscribe to migration status changes
    - Update display without page refresh
    - _Requirements: 16.2_

- [ ] 16. Implement documentation system
  - [ ] 16.1 Set up markdown content structure
    - Create content/docs directory with markdown files
    - Organize by sections: installation, quick-start, commands, configuration, API, migrations, troubleshooting
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 20.1_
  
  - [ ] 16.2 Create documentation layout and navigation
    - Implement app/docs/layout.tsx with sidebar navigation
    - Generate navigation tree from file structure
    - _Requirements: 1.9, 1.10_
  
  - [ ] 16.3 Implement markdown rendering
    - Set up next-mdx-remote or similar library
    - Add syntax highlighting for code blocks
    - _Requirements: 1.8, 20.2_
  
  - [ ] 16.4 Create documentation components
    - Implement DocsContent with table of contents generation
    - Add CodeBlock component with copy functionality
    - _Requirements: 20.3, 20.4_
  
  - [ ] 16.5 Add documentation search
    - Implement search functionality across all docs
    - _Requirements: 1.9_
  
  - [ ] 16.6 Write documentation content
    - Write installation guide with platform-specific instructions
    - Write quick start tutorial with examples
    - Write command reference for all CLI commands
    - Write configuration documentation
    - Write API documentation
    - Write migration guides for supported language pairs
    - Write troubleshooting guides
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8_

- [ ] 17. Implement footer pages
  - [ ] 17.1 Create privacy policy page
    - Write content describing data collection, storage, usage
    - Implement app/privacy/page.tsx
    - _Requirements: 2.1_
  
  - [ ] 17.2 Create terms of service page
    - Write content defining user rights, responsibilities, limitations
    - Implement app/terms/page.tsx
    - _Requirements: 2.2_
  
  - [ ] 17.3 Create blog system
    - Set up content/blog directory for markdown posts
    - Implement app/blog/page.tsx with post listing
    - Implement app/blog/[slug]/page.tsx for individual posts
    - Add frontmatter support for metadata
    - Display publish dates and authors
    - Add categorization and tagging
    - Generate RSS feed
    - _Requirements: 2.3, 20.5, 20.6, 20.7, 20.8_
  
  - [ ] 17.4 Create careers page
    - Write content about open positions and company culture
    - Implement app/careers/page.tsx
    - _Requirements: 2.4_
  
  - [ ] 17.5 Create contact page with form
    - Implement app/contact/page.tsx with contact form
    - Add form validation for required fields
    - Create POST /api/contact route for email delivery
    - Display confirmation message on success
    - _Requirements: 2.5, 2.6, 2.7_
  
  - [ ] 17.6 Create integrations page
    - Write content listing IDE, CI/CD, and tool integrations
    - Implement app/integrations/page.tsx
    - _Requirements: 2.8_
  
  - [ ] 17.7 Create pricing page
    - Write content for pricing tiers with feature comparisons
    - Add call-to-action buttons for each tier
    - Implement app/pricing/page.tsx
    - _Requirements: 2.9, 2.10_

- [ ] 18. Checkpoint - Verify all content pages complete
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 19. Implement performance optimizations
  - [ ] 19.1 Add pagination to all list views
    - Implement pagination for migrations, tasks, timeline events
    - Limit to 50 items per page
    - _Requirements: 17.2_
  
  - [ ] 19.2 Implement lazy loading
    - Lazy load images and heavy components
    - Use Next.js dynamic imports
    - _Requirements: 17.3_
  
  - [ ] 19.3 Add caching layer
    - Cache frequently accessed data in browser storage
    - Implement cache invalidation strategy
    - _Requirements: 17.4_
  
  - [ ] 19.4 Implement prefetching
    - Prefetch data for likely next navigation targets
    - _Requirements: 17.5_
  
  - [ ] 19.5 Add input debouncing
    - Debounce search and filter inputs to reduce API calls
    - _Requirements: 17.6_
  
  - [ ] 19.6 Implement optimistic updates
    - Use optimistic updates for mutations
    - Revert on error
    - _Requirements: 17.7_
  
  - [ ] 19.7 Optimize bundle size
    - Implement code splitting
    - Analyze and reduce bundle size
    - _Requirements: 17.8_

- [ ] 20. Implement responsive design and polish
  - [ ] 20.1 Ensure responsive layouts
    - Test all pages on mobile, tablet, desktop viewports
    - Fix any layout issues
    - _Requirements: 15.1, 15.7_
  
  - [ ] 20.2 Add animations and transitions
    - Implement simple fade-in animations for page transitions
    - Add hover states for interactive elements
    - _Requirements: 15.2, 15.6, 15.8_
  
  - [ ] 20.3 Apply consistent design system
    - Ensure square borders throughout
    - Use Cabinet Grotesk font for headings
    - Apply consistent spacing and colors
    - _Requirements: 15.3, 15.4, 15.5, 15.9_
  
  - [ ] 20.4 Ensure accessibility
    - Verify keyboard navigation works for all interactive elements
    - Test with screen readers
    - _Requirements: 15.10_

- [ ] 21. Implement real-time connection management
  - [ ] 21.1 Create real-time connection utilities
    - Implement connection status tracking
    - Add automatic reconnection logic
    - _Requirements: 16.5, 16.6_
  
  - [ ] 21.2 Add connection status indicator
    - Display connection status in dashboard
    - _Requirements: 16.8_
  
  - [ ] 21.3 Implement sync on reconnection
    - Sync missed updates when connection restored
    - _Requirements: 16.7_
  
  - [ ] 21.4 Add real-time settings control
    - Allow users to disable real-time updates in settings
    - Limit subscriptions to active pages
    - _Requirements: 16.9, 16.10_

- [ ] 22. Final integration and testing
  - [ ] 22.1 Test complete authentication flow
    - Verify sign-in, sign-out, session management
    - Test expired session handling
    - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.6, 13.9, 13.10_
  
  - [ ] 22.2 Test all data loading and error states
    - Verify loading indicators display correctly
    - Test error handling and retry functionality
    - Verify success messages
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5, 14.8, 14.9_
  
  - [ ] 22.3 Test real-time updates across all features
    - Verify subscriptions work for migrations, tasks, health metrics
    - Test connection failure and recovery
    - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5, 16.6, 16.7_
  
  - [ ] 22.4 Verify performance targets
    - Test page load times (target: under 2 seconds)
    - Verify pagination, lazy loading, caching work correctly
    - Measure Core Web Vitals
    - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5, 17.6, 17.7, 17.8, 17.9, 17.10_
  
  - [ ] 22.5 Cross-browser and device testing
    - Test on Chrome, Firefox, Safari, Edge
    - Test on mobile, tablet, desktop viewports
    - Verify responsive design and accessibility
    - _Requirements: 15.1, 15.7, 15.10_

- [ ] 23. Final checkpoint - Complete system verification
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation at key milestones
- Real-time features are marked optional as they can be added after core functionality
- The implementation follows a bottom-up approach: infrastructure first, then features, then polish
- All database queries automatically enforce Row Level Security for user data isolation
- Performance optimizations should be implemented incrementally and measured
