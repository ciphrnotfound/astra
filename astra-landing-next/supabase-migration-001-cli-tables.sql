-- Astra Content and Backend Integration Migration
-- Migration 001: CLI-Synced Tables
-- Run this migration AFTER the base schema (supabase-schema.sql)
-- 
-- This migration creates all tables that receive data from the Rust CLI
-- These tables store data synced from developer machines running the Astra CLI

-- ============================================================================
-- CLI-SYNCED TABLES (Data from Rust CLI)
-- ============================================================================

-- Astra Sessions - synced from CLI teams.rs
-- Tracks developer work sessions with code metrics
-- Note: project_id is nullable because CLI doesn't currently send it
CREATE TABLE IF NOT EXISTS astra_sessions (
  id SERIAL PRIMARY KEY,
  project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
  task_id VARCHAR(255) NOT NULL,
  developer VARCHAR(255) NOT NULL,
  start_time BIGINT NOT NULL,
  end_time BIGINT NOT NULL,
  lines_added INTEGER NOT NULL DEFAULT 0,
  lines_deleted INTEGER NOT NULL DEFAULT 0,
  prompts_asked JSONB DEFAULT '[]'::jsonb,
  files_touched JSONB DEFAULT '[]'::jsonb,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_astra_sessions_project_id ON astra_sessions(project_id);
CREATE INDEX IF NOT EXISTS idx_astra_sessions_developer ON astra_sessions(developer);
CREATE INDEX IF NOT EXISTS idx_astra_sessions_task_id ON astra_sessions(task_id);
CREATE INDEX IF NOT EXISTS idx_astra_sessions_start_time ON astra_sessions(start_time DESC);

-- Health Snapshots - synced from CLI health.rs
-- Stores codebase health metrics over time
CREATE TABLE IF NOT EXISTS health_snapshots (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  code_quality INTEGER NOT NULL CHECK (code_quality >= 0 AND code_quality <= 100),
  test_health INTEGER NOT NULL CHECK (test_health >= 0 AND test_health <= 100),
  cross_lang_drift INTEGER NOT NULL CHECK (cross_lang_drift >= 0 AND cross_lang_drift <= 100),
  security_surface INTEGER NOT NULL CHECK (security_surface >= 0 AND security_surface <= 100),
  git_health INTEGER NOT NULL CHECK (git_health >= 0 AND git_health <= 100),
  team_velocity INTEGER NOT NULL CHECK (team_velocity >= 0 AND team_velocity <= 100),
  total_lines INTEGER NOT NULL DEFAULT 0,
  file_count INTEGER NOT NULL DEFAULT 0,
  todo_count INTEGER NOT NULL DEFAULT 0,
  test_files INTEGER NOT NULL DEFAULT 0,
  language_count INTEGER NOT NULL DEFAULT 0,
  migration_count INTEGER NOT NULL DEFAULT 0,
  security_files INTEGER NOT NULL DEFAULT 0,
  uncommitted_changes INTEGER NOT NULL DEFAULT 0,
  recent_commits INTEGER NOT NULL DEFAULT 0,
  tasks_done INTEGER NOT NULL DEFAULT 0,
  tasks_total INTEGER NOT NULL DEFAULT 0,
  timestamp BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_health_snapshots_project_id ON health_snapshots(project_id);
CREATE INDEX IF NOT EXISTS idx_health_snapshots_timestamp ON health_snapshots(timestamp DESC);

-- Security Issues - synced from CLI security.rs
-- Tracks security vulnerabilities found in codebase
CREATE TABLE IF NOT EXISTS security_issues (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  severity VARCHAR(20) NOT NULL CHECK (severity IN ('Critical', 'High', 'Medium', 'Low')),
  file_path TEXT NOT NULL,
  line_number INTEGER NOT NULL,
  description TEXT NOT NULL,
  snippet TEXT NOT NULL,
  status VARCHAR(20) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved', 'ignored')),
  detected_at BIGINT NOT NULL,
  resolved_at BIGINT,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_security_issues_project_id ON security_issues(project_id);
CREATE INDEX IF NOT EXISTS idx_security_issues_severity ON security_issues(severity);
CREATE INDEX IF NOT EXISTS idx_security_issues_status ON security_issues(status);
CREATE INDEX IF NOT EXISTS idx_security_issues_detected_at ON security_issues(detected_at DESC);

-- Timeline Events - synced from CLI memory.rs
-- Records significant events in codebase history
CREATE TABLE IF NOT EXISTS timeline_events (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  event_type VARCHAR(50) NOT NULL,
  content TEXT NOT NULL,
  event_data JSONB,
  timestamp BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_timeline_events_project_id ON timeline_events(project_id);
CREATE INDEX IF NOT EXISTS idx_timeline_events_type ON timeline_events(event_type);
CREATE INDEX IF NOT EXISTS idx_timeline_events_timestamp ON timeline_events(timestamp DESC);

-- Dependencies - synced from CLI index.rs
-- Stores code dependency graph data
CREATE TABLE IF NOT EXISTS dependencies (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  source_file TEXT NOT NULL,
  target_file TEXT NOT NULL,
  dependency_type VARCHAR(50) NOT NULL,
  metadata JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, source_file, target_file, dependency_type)
);

CREATE INDEX IF NOT EXISTS idx_dependencies_project_id ON dependencies(project_id);
CREATE INDEX IF NOT EXISTS idx_dependencies_source_file ON dependencies(source_file);
CREATE INDEX IF NOT EXISTS idx_dependencies_target_file ON dependencies(target_file);

-- Tasks - synced from CLI teams.rs
-- Tracks team tasks and assignments
CREATE TABLE IF NOT EXISTS tasks (
  id SERIAL PRIMARY KEY,
  project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
  task_id VARCHAR(255) NOT NULL UNIQUE,
  description TEXT NOT NULL,
  assignee VARCHAR(255) NOT NULL,
  status VARCHAR(20) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'InProgress', 'Done')),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee ON tasks(assignee);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_task_id ON tasks(task_id);

-- ============================================================================
-- DASHBOARD-MANAGED TABLES (Data managed by Next.js dashboard)
-- ============================================================================

-- Learning Phases - onboarding content for junior developers
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

-- User Progress - tracks learning completion
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

-- User Settings - user preferences and configuration
CREATE TABLE IF NOT EXISTS user_settings (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
  persona JSONB DEFAULT '{"role": "developer", "experience": "intermediate", "preferences": {}}'::jsonb,
  model_config JSONB DEFAULT '{"model": "gpt-4", "temperature": 0.7, "max_tokens": 2000}'::jsonb,
  integrations JSONB DEFAULT '{}'::jsonb,
  notifications JSONB DEFAULT '{"email": true, "realtime": true}'::jsonb,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_settings_user_id ON user_settings(user_id);

-- Vulnerabilities (legacy table for dashboard-managed vulnerabilities)
-- Note: This is separate from security_issues which is CLI-synced
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
-- MIGRATION COMPLETE
-- ============================================================================
-- All CLI-synced and dashboard-managed tables have been created
-- 
-- CLI-Synced Tables (read-only from dashboard):
-- - astra_sessions
-- - health_snapshots
-- - security_issues
-- - timeline_events
-- - dependencies
-- - tasks
-- 
-- Dashboard-Managed Tables (read-write from dashboard):
-- - learning_phases
-- - user_progress
-- - user_settings
-- - vulnerabilities (legacy)
-- 
-- Next steps:
-- 1. Run migration 002 to enable Row Level Security (RLS)
-- 2. Configure Rust CLI to sync data to these tables
-- 3. Update dashboard to read from these tables

