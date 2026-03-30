-- Astra Content and Backend Integration Migration
-- Migration 001: Add new tables for vulnerabilities, tasks, timeline events, learning phases, user progress, dependencies, and user settings
-- Run this migration after the base schema (supabase-schema.sql)

-- ============================================================================
-- VULNERABILITIES TABLE
-- Stores security vulnerabilities detected in projects
-- ============================================================================
CREATE TABLE IF NOT EXISTS vulnerabilities (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  severity VARCHAR(20) NOT NULL CHECK (severity IN ('critical', 'high', 'medium', 'low')),
  title VARCHAR(255) NOT NULL,
  description TEXT,
  file_path TEXT NOT NULL,
  line_number INTEGER,
  cwe_id VARCHAR(20),
  remediation TEXT,
  status VARCHAR(20) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved', 'ignored')),
  detected_at TIMESTAMP NOT NULL DEFAULT NOW(),
  resolved_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vulnerabilities_project_id ON vulnerabilities(project_id);
CREATE INDEX IF NOT EXISTS idx_vulnerabilities_severity ON vulnerabilities(severity);
CREATE INDEX IF NOT EXISTS idx_vulnerabilities_status ON vulnerabilities(status);

-- ============================================================================
-- TASKS TABLE
-- Stores development tasks for team collaboration
-- ============================================================================
CREATE TABLE IF NOT EXISTS tasks (
  id SERIAL PRIMARY KEY,
  project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
  title VARCHAR(255) NOT NULL,
  description TEXT,
  status VARCHAR(20) NOT NULL DEFAULT 'todo' CHECK (status IN ('todo', 'in_progress', 'done')),
  priority VARCHAR(20) DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
  assignee_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
  created_by INTEGER NOT NULL REFERENCES users(id),
  tags TEXT[],
  due_date TIMESTAMP,
  completed_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee_id ON tasks(assignee_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC);

-- ============================================================================
-- TIMELINE_EVENTS TABLE
-- Stores codebase memory timeline events
-- ============================================================================
CREATE TABLE IF NOT EXISTS timeline_events (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  event_type VARCHAR(50) NOT NULL CHECK (event_type IN ('analysis', 'migration', 'refactor', 'security_scan', 'deployment')),
  title VARCHAR(255) NOT NULL,
  description TEXT,
  affected_files TEXT[],
  metadata JSONB,
  user_id INTEGER REFERENCES users(id),
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_timeline_events_project_id ON timeline_events(project_id);
CREATE INDEX IF NOT EXISTS idx_timeline_events_type ON timeline_events(event_type);
CREATE INDEX IF NOT EXISTS idx_timeline_events_created_at ON timeline_events(created_at DESC);

-- ============================================================================
-- LEARNING_PHASES TABLE
-- Stores onboarding learning phases content
-- ============================================================================
CREATE TABLE IF NOT EXISTS learning_phases (
  id SERIAL PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  description TEXT,
  content TEXT NOT NULL,
  order_index INTEGER NOT NULL,
  estimated_minutes INTEGER,
  prerequisites INTEGER[],
  exercises JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_learning_phases_order ON learning_phases(order_index);

-- ============================================================================
-- USER_PROGRESS TABLE
-- Tracks user progress through learning phases
-- ============================================================================
CREATE TABLE IF NOT EXISTS user_progress (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  phase_id INTEGER NOT NULL REFERENCES learning_phases(id) ON DELETE CASCADE,
  status VARCHAR(20) NOT NULL DEFAULT 'not_started' CHECK (status IN ('not_started', 'in_progress', 'completed')),
  progress_percentage INTEGER DEFAULT 0 CHECK (progress_percentage >= 0 AND progress_percentage <= 100),
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  UNIQUE(user_id, phase_id)
);

CREATE INDEX IF NOT EXISTS idx_user_progress_user_id ON user_progress(user_id);
CREATE INDEX IF NOT EXISTS idx_user_progress_phase_id ON user_progress(phase_id);

-- ============================================================================
-- DEPENDENCIES TABLE
-- Stores code dependency relationships for semantic cartographer
-- ============================================================================
CREATE TABLE IF NOT EXISTS dependencies (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  source_file TEXT NOT NULL,
  target_file TEXT NOT NULL,
  dependency_type VARCHAR(50) NOT NULL,
  line_number INTEGER,
  metadata JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, source_file, target_file, dependency_type)
);

CREATE INDEX IF NOT EXISTS idx_dependencies_project_id ON dependencies(project_id);
CREATE INDEX IF NOT EXISTS idx_dependencies_source_file ON dependencies(source_file);
CREATE INDEX IF NOT EXISTS idx_dependencies_target_file ON dependencies(target_file);

-- ============================================================================
-- USER_SETTINGS TABLE
-- Stores user preferences and configuration
-- ============================================================================
CREATE TABLE IF NOT EXISTS user_settings (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
  persona JSONB DEFAULT '{"role": "developer", "experience": "intermediate", "preferences": {}}',
  model_config JSONB DEFAULT '{"model": "gpt-4", "temperature": 0.7, "max_tokens": 2000}',
  integrations JSONB DEFAULT '{}',
  notifications JSONB DEFAULT '{"email": true, "realtime": true}',
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_settings_user_id ON user_settings(user_id);

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
-- All tables created successfully
-- Next steps:
-- 1. Run RLS policies migration (supabase-migration-002-rls.sql)
-- 2. Update TypeScript types (lib/db/types.ts)
-- 3. Create query utilities (lib/db/queries.ts)
