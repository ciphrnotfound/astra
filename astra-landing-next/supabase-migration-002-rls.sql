-- Astra Content and Backend Integration Migration
-- Migration 002: Row Level Security (RLS) Policies
-- Run this migration after migration 001 (supabase-migration-001-content-backend.sql)
-- 
-- This migration implements Row Level Security policies to ensure user data isolation.
-- All queries will automatically enforce these policies based on the authenticated user.

-- ============================================================================
-- ENABLE ROW LEVEL SECURITY ON ALL TABLES
-- ============================================================================

ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE migrations ENABLE ROW LEVEL SECURITY;
ALTER TABLE codebase_analytics ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE vulnerabilities ENABLE ROW LEVEL SECURITY;
ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE timeline_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE learning_phases ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_progress ENABLE ROW LEVEL SECURITY;
ALTER TABLE dependencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_settings ENABLE ROW LEVEL SECURITY;

-- ============================================================================
-- USERS TABLE POLICIES
-- Users can view and update their own profile
-- ============================================================================

CREATE POLICY "Users can view own profile"
  ON users FOR SELECT
  USING (auth.uid()::text = id::text);

CREATE POLICY "Users can update own profile"
  ON users FOR UPDATE
  USING (auth.uid()::text = id::text);

-- ============================================================================
-- PROJECTS TABLE POLICIES
-- Users can only access their own projects
-- ============================================================================

CREATE POLICY "Users can view own projects"
  ON projects FOR SELECT
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can insert own projects"
  ON projects FOR INSERT
  WITH CHECK (auth.uid()::text = user_id::text);

CREATE POLICY "Users can update own projects"
  ON projects FOR UPDATE
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can delete own projects"
  ON projects FOR DELETE
  USING (auth.uid()::text = user_id::text);

-- ============================================================================
-- MIGRATIONS TABLE POLICIES
-- Users can only access migrations for their projects
-- ============================================================================

CREATE POLICY "Users can view own migrations"
  ON migrations FOR SELECT
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can insert own migrations"
  ON migrations FOR INSERT
  WITH CHECK (auth.uid()::text = user_id::text);

CREATE POLICY "Users can update own migrations"
  ON migrations FOR UPDATE
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can delete own migrations"
  ON migrations FOR DELETE
  USING (auth.uid()::text = user_id::text);

-- ============================================================================
-- CODEBASE_ANALYTICS TABLE POLICIES
-- Users can access analytics for their projects
-- ============================================================================

CREATE POLICY "Users can view project analytics"
  ON codebase_analytics FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = codebase_analytics.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can insert project analytics"
  ON codebase_analytics FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = codebase_analytics.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can update project analytics"
  ON codebase_analytics FOR UPDATE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = codebase_analytics.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can delete project analytics"
  ON codebase_analytics FOR DELETE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = codebase_analytics.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

-- ============================================================================
-- API_KEYS TABLE POLICIES
-- Users can only access their own API keys
-- ============================================================================

CREATE POLICY "Users can view own api keys"
  ON api_keys FOR SELECT
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can insert own api keys"
  ON api_keys FOR INSERT
  WITH CHECK (auth.uid()::text = user_id::text);

CREATE POLICY "Users can update own api keys"
  ON api_keys FOR UPDATE
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can delete own api keys"
  ON api_keys FOR DELETE
  USING (auth.uid()::text = user_id::text);

-- ============================================================================
-- VULNERABILITIES TABLE POLICIES
-- Users can access vulnerabilities for their projects
-- ============================================================================

CREATE POLICY "Users can view project vulnerabilities"
  ON vulnerabilities FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = vulnerabilities.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can insert project vulnerabilities"
  ON vulnerabilities FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = vulnerabilities.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can update project vulnerabilities"
  ON vulnerabilities FOR UPDATE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = vulnerabilities.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can delete project vulnerabilities"
  ON vulnerabilities FOR DELETE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = vulnerabilities.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

-- ============================================================================
-- TASKS TABLE POLICIES
-- Users can access tasks they created or are assigned to
-- ============================================================================

CREATE POLICY "Users can view relevant tasks"
  ON tasks FOR SELECT
  USING (
    auth.uid()::text = created_by::text
    OR auth.uid()::text = assignee_id::text
    OR (
      project_id IS NOT NULL
      AND EXISTS (
        SELECT 1 FROM projects
        WHERE projects.id = tasks.project_id
        AND projects.user_id::text = auth.uid()::text
      )
    )
  );

CREATE POLICY "Users can insert tasks"
  ON tasks FOR INSERT
  WITH CHECK (
    auth.uid()::text = created_by::text
    AND (
      project_id IS NULL
      OR EXISTS (
        SELECT 1 FROM projects
        WHERE projects.id = tasks.project_id
        AND projects.user_id::text = auth.uid()::text
      )
    )
  );

CREATE POLICY "Users can update relevant tasks"
  ON tasks FOR UPDATE
  USING (
    auth.uid()::text = created_by::text
    OR auth.uid()::text = assignee_id::text
    OR (
      project_id IS NOT NULL
      AND EXISTS (
        SELECT 1 FROM projects
        WHERE projects.id = tasks.project_id
        AND projects.user_id::text = auth.uid()::text
      )
    )
  );

CREATE POLICY "Users can delete own tasks"
  ON tasks FOR DELETE
  USING (
    auth.uid()::text = created_by::text
    OR (
      project_id IS NOT NULL
      AND EXISTS (
        SELECT 1 FROM projects
        WHERE projects.id = tasks.project_id
        AND projects.user_id::text = auth.uid()::text
      )
    )
  );

-- ============================================================================
-- TIMELINE_EVENTS TABLE POLICIES
-- Users can access timeline events for their projects
-- ============================================================================

CREATE POLICY "Users can view project timeline events"
  ON timeline_events FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = timeline_events.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can insert project timeline events"
  ON timeline_events FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = timeline_events.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can update project timeline events"
  ON timeline_events FOR UPDATE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = timeline_events.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can delete project timeline events"
  ON timeline_events FOR DELETE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = timeline_events.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

-- ============================================================================
-- LEARNING_PHASES TABLE POLICIES
-- Learning phases are public content - all authenticated users can read
-- Only admins can modify (not implemented in this migration)
-- ============================================================================

CREATE POLICY "Authenticated users can view learning phases"
  ON learning_phases FOR SELECT
  USING (auth.uid() IS NOT NULL);

-- Note: INSERT, UPDATE, DELETE policies for learning_phases should be restricted
-- to admin users. This can be implemented later with a role-based system.

-- ============================================================================
-- USER_PROGRESS TABLE POLICIES
-- Users can only access their own learning progress
-- ============================================================================

CREATE POLICY "Users can view own progress"
  ON user_progress FOR SELECT
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can insert own progress"
  ON user_progress FOR INSERT
  WITH CHECK (auth.uid()::text = user_id::text);

CREATE POLICY "Users can update own progress"
  ON user_progress FOR UPDATE
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can delete own progress"
  ON user_progress FOR DELETE
  USING (auth.uid()::text = user_id::text);

-- ============================================================================
-- DEPENDENCIES TABLE POLICIES
-- Users can access dependencies for their projects
-- ============================================================================

CREATE POLICY "Users can view project dependencies"
  ON dependencies FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = dependencies.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can insert project dependencies"
  ON dependencies FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = dependencies.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can update project dependencies"
  ON dependencies FOR UPDATE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = dependencies.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

CREATE POLICY "Users can delete project dependencies"
  ON dependencies FOR DELETE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = dependencies.project_id
      AND projects.user_id::text = auth.uid()::text
    )
  );

-- ============================================================================
-- USER_SETTINGS TABLE POLICIES
-- Users can only access their own settings
-- ============================================================================

CREATE POLICY "Users can view own settings"
  ON user_settings FOR SELECT
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can insert own settings"
  ON user_settings FOR INSERT
  WITH CHECK (auth.uid()::text = user_id::text);

CREATE POLICY "Users can update own settings"
  ON user_settings FOR UPDATE
  USING (auth.uid()::text = user_id::text);

CREATE POLICY "Users can delete own settings"
  ON user_settings FOR DELETE
  USING (auth.uid()::text = user_id::text);

-- ============================================================================
-- RLS POLICIES MIGRATION COMPLETE
-- ============================================================================
-- All tables now have Row Level Security enabled with appropriate policies
-- 
-- Summary:
-- - Users can only access their own data (users, projects, migrations, api_keys, user_settings, user_progress)
-- - Users can access data related to their projects (codebase_analytics, vulnerabilities, timeline_events, dependencies)
-- - Users can access tasks they created, are assigned to, or belong to their projects
-- - Learning phases are readable by all authenticated users (content is public)
-- 
-- Next steps:
-- 1. Test policies with different user contexts
-- 2. Update TypeScript types (lib/db/types.ts)
-- 3. Create query utilities (lib/db/queries.ts)
