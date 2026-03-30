-- RLS Policy Testing Script
-- This script tests Row Level Security policies with different user contexts
-- Run this in Supabase SQL Editor after applying migration 002

-- ============================================================================
-- SETUP TEST DATA
-- ============================================================================

-- Create test users
INSERT INTO users (id, name, email, password_hash, role) VALUES
  (1, 'Alice Developer', 'alice@example.com', 'hash1', 'member'),
  (2, 'Bob Developer', 'bob@example.com', 'hash2', 'member')
ON CONFLICT (id) DO NOTHING;

-- Create test projects
INSERT INTO projects (id, user_id, name, description, repository_url) VALUES
  (1, 1, 'Alice Project', 'Alice''s test project', 'https://github.com/alice/project'),
  (2, 2, 'Bob Project', 'Bob''s test project', 'https://github.com/bob/project')
ON CONFLICT (id) DO NOTHING;

-- Create test migrations
INSERT INTO migrations (id, project_id, user_id, source_language, target_language, status) VALUES
  (1, 1, 1, 'JavaScript', 'TypeScript', 'completed'),
  (2, 2, 2, 'Python', 'Go', 'in_progress')
ON CONFLICT (id) DO NOTHING;

-- Create test vulnerabilities
INSERT INTO vulnerabilities (id, project_id, severity, title, file_path, status) VALUES
  (1, 1, 'high', 'SQL Injection', '/src/db.js', 'open'),
  (2, 2, 'medium', 'XSS Vulnerability', '/src/render.py', 'open')
ON CONFLICT (id) DO NOTHING;

-- Create test tasks
INSERT INTO tasks (id, project_id, title, status, created_by, assignee_id) VALUES
  (1, 1, 'Fix SQL injection', 'todo', 1, 1),
  (2, 2, 'Fix XSS', 'in_progress', 2, 2),
  (3, NULL, 'Personal task for Alice', 'todo', 1, 1)
ON CONFLICT (id) DO NOTHING;

-- Create test timeline events
INSERT INTO timeline_events (id, project_id, event_type, title, user_id) VALUES
  (1, 1, 'security_scan', 'Security scan completed', 1),
  (2, 2, 'migration', 'Migration started', 2)
ON CONFLICT (id) DO NOTHING;

-- Create test learning phases
INSERT INTO learning_phases (id, title, content, order_index) VALUES
  (1, 'Getting Started', 'Introduction to Astra', 1),
  (2, 'Advanced Features', 'Deep dive into features', 2)
ON CONFLICT (id) DO NOTHING;

-- Create test user progress
INSERT INTO user_progress (user_id, phase_id, status, progress_percentage) VALUES
  (1, 1, 'completed', 100),
  (2, 1, 'in_progress', 50)
ON CONFLICT (user_id, phase_id) DO NOTHING;

-- Create test dependencies
INSERT INTO dependencies (id, project_id, source_file, target_file, dependency_type) VALUES
  (1, 1, '/src/index.js', '/src/utils.js', 'import'),
  (2, 2, '/src/main.py', '/src/helpers.py', 'import')
ON CONFLICT (id) DO NOTHING;

-- Create test user settings
INSERT INTO user_settings (user_id, persona, model_config) VALUES
  (1, '{"role": "senior", "experience": "expert"}', '{"model": "gpt-4"}'),
  (2, '{"role": "junior", "experience": "beginner"}', '{"model": "gpt-3.5"}')
ON CONFLICT (user_id) DO NOTHING;

-- Create test codebase analytics
INSERT INTO codebase_analytics (project_id, lines_of_code, files_count, security_score) VALUES
  (1, 10000, 50, 85),
  (2, 5000, 30, 90)
ON CONFLICT DO NOTHING;

-- Create test API keys
INSERT INTO api_keys (user_id, name, key_hash, key_prefix) VALUES
  (1, 'Alice API Key', 'hash_alice', 'ak_alice'),
  (2, 'Bob API Key', 'hash_bob', 'ak_bob')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- TEST CASES
-- ============================================================================

-- Test 1: Users can only view their own profile
-- Expected: Alice (user 1) can see her profile but not Bob's
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see her own profile
  IF NOT EXISTS (SELECT 1 FROM users WHERE id = 1) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see her own profile';
  END IF;
  
  -- Alice should NOT see Bob's profile
  IF EXISTS (SELECT 1 FROM users WHERE id = 2) THEN
    RAISE EXCEPTION 'FAIL: Alice can see Bob''s profile (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: Users table RLS working correctly';
END $$;

-- Test 2: Users can only view their own projects
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see her own project
  IF NOT EXISTS (SELECT 1 FROM projects WHERE id = 1) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see her own project';
  END IF;
  
  -- Alice should NOT see Bob's project
  IF EXISTS (SELECT 1 FROM projects WHERE id = 2) THEN
    RAISE EXCEPTION 'FAIL: Alice can see Bob''s project (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: Projects table RLS working correctly';
END $$;

-- Test 3: Users can only view their own migrations
DO $$
BEGIN
  -- Simulate Bob's session
  PERFORM set_config('request.jwt.claims', '{"sub": "2"}', true);
  
  -- Bob should see his own migration
  IF NOT EXISTS (SELECT 1 FROM migrations WHERE id = 2) THEN
    RAISE EXCEPTION 'FAIL: Bob cannot see his own migration';
  END IF;
  
  -- Bob should NOT see Alice's migration
  IF EXISTS (SELECT 1 FROM migrations WHERE id = 1) THEN
    RAISE EXCEPTION 'FAIL: Bob can see Alice''s migration (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: Migrations table RLS working correctly';
END $$;

-- Test 4: Users can view vulnerabilities for their projects
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see vulnerabilities in her project
  IF NOT EXISTS (SELECT 1 FROM vulnerabilities WHERE id = 1) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see vulnerabilities in her project';
  END IF;
  
  -- Alice should NOT see vulnerabilities in Bob's project
  IF EXISTS (SELECT 1 FROM vulnerabilities WHERE id = 2) THEN
    RAISE EXCEPTION 'FAIL: Alice can see vulnerabilities in Bob''s project (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: Vulnerabilities table RLS working correctly';
END $$;

-- Test 5: Users can view tasks they created or are assigned to
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see her own tasks
  IF NOT EXISTS (SELECT 1 FROM tasks WHERE id = 1) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see her own task';
  END IF;
  
  IF NOT EXISTS (SELECT 1 FROM tasks WHERE id = 3) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see her personal task';
  END IF;
  
  -- Alice should NOT see Bob's task
  IF EXISTS (SELECT 1 FROM tasks WHERE id = 2) THEN
    RAISE EXCEPTION 'FAIL: Alice can see Bob''s task (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: Tasks table RLS working correctly';
END $$;

-- Test 6: Users can view timeline events for their projects
DO $$
BEGIN
  -- Simulate Bob's session
  PERFORM set_config('request.jwt.claims', '{"sub": "2"}', true);
  
  -- Bob should see timeline events in his project
  IF NOT EXISTS (SELECT 1 FROM timeline_events WHERE id = 2) THEN
    RAISE EXCEPTION 'FAIL: Bob cannot see timeline events in his project';
  END IF;
  
  -- Bob should NOT see timeline events in Alice's project
  IF EXISTS (SELECT 1 FROM timeline_events WHERE id = 1) THEN
    RAISE EXCEPTION 'FAIL: Bob can see timeline events in Alice''s project (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: Timeline events table RLS working correctly';
END $$;

-- Test 7: All authenticated users can view learning phases
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see all learning phases
  IF (SELECT COUNT(*) FROM learning_phases) != 2 THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see all learning phases';
  END IF;
  
  -- Simulate Bob's session
  PERFORM set_config('request.jwt.claims', '{"sub": "2"}', true);
  
  -- Bob should also see all learning phases
  IF (SELECT COUNT(*) FROM learning_phases) != 2 THEN
    RAISE EXCEPTION 'FAIL: Bob cannot see all learning phases';
  END IF;
  
  RAISE NOTICE 'PASS: Learning phases table RLS working correctly';
END $$;

-- Test 8: Users can only view their own progress
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see her own progress
  IF NOT EXISTS (SELECT 1 FROM user_progress WHERE user_id = 1) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see her own progress';
  END IF;
  
  -- Alice should NOT see Bob's progress
  IF EXISTS (SELECT 1 FROM user_progress WHERE user_id = 2) THEN
    RAISE EXCEPTION 'FAIL: Alice can see Bob''s progress (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: User progress table RLS working correctly';
END $$;

-- Test 9: Users can view dependencies for their projects
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see dependencies in her project
  IF NOT EXISTS (SELECT 1 FROM dependencies WHERE id = 1) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see dependencies in her project';
  END IF;
  
  -- Alice should NOT see dependencies in Bob's project
  IF EXISTS (SELECT 1 FROM dependencies WHERE id = 2) THEN
    RAISE EXCEPTION 'FAIL: Alice can see dependencies in Bob''s project (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: Dependencies table RLS working correctly';
END $$;

-- Test 10: Users can only view their own settings
DO $$
BEGIN
  -- Simulate Bob's session
  PERFORM set_config('request.jwt.claims', '{"sub": "2"}', true);
  
  -- Bob should see his own settings
  IF NOT EXISTS (SELECT 1 FROM user_settings WHERE user_id = 2) THEN
    RAISE EXCEPTION 'FAIL: Bob cannot see his own settings';
  END IF;
  
  -- Bob should NOT see Alice's settings
  IF EXISTS (SELECT 1 FROM user_settings WHERE user_id = 1) THEN
    RAISE EXCEPTION 'FAIL: Bob can see Alice''s settings (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: User settings table RLS working correctly';
END $$;

-- Test 11: Users can view analytics for their projects
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see analytics for her project
  IF NOT EXISTS (SELECT 1 FROM codebase_analytics WHERE project_id = 1) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see analytics for her project';
  END IF;
  
  -- Alice should NOT see analytics for Bob's project
  IF EXISTS (SELECT 1 FROM codebase_analytics WHERE project_id = 2) THEN
    RAISE EXCEPTION 'FAIL: Alice can see analytics for Bob''s project (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: Codebase analytics table RLS working correctly';
END $$;

-- Test 12: Users can only view their own API keys
DO $$
BEGIN
  -- Simulate Alice's session
  PERFORM set_config('request.jwt.claims', '{"sub": "1"}', true);
  
  -- Alice should see her own API keys
  IF NOT EXISTS (SELECT 1 FROM api_keys WHERE user_id = 1) THEN
    RAISE EXCEPTION 'FAIL: Alice cannot see her own API keys';
  END IF;
  
  -- Alice should NOT see Bob's API keys
  IF EXISTS (SELECT 1 FROM api_keys WHERE user_id = 2) THEN
    RAISE EXCEPTION 'FAIL: Alice can see Bob''s API keys (should be blocked)';
  END IF;
  
  RAISE NOTICE 'PASS: API keys table RLS working correctly';
END $$;

-- ============================================================================
-- TEST SUMMARY
-- ============================================================================

DO $$
BEGIN
  RAISE NOTICE '========================================';
  RAISE NOTICE 'RLS POLICY TESTING COMPLETE';
  RAISE NOTICE '========================================';
  RAISE NOTICE 'All 12 test cases passed successfully!';
  RAISE NOTICE '';
  RAISE NOTICE 'Tested tables:';
  RAISE NOTICE '  ✓ users';
  RAISE NOTICE '  ✓ projects';
  RAISE NOTICE '  ✓ migrations';
  RAISE NOTICE '  ✓ vulnerabilities';
  RAISE NOTICE '  ✓ tasks';
  RAISE NOTICE '  ✓ timeline_events';
  RAISE NOTICE '  ✓ learning_phases';
  RAISE NOTICE '  ✓ user_progress';
  RAISE NOTICE '  ✓ dependencies';
  RAISE NOTICE '  ✓ user_settings';
  RAISE NOTICE '  ✓ codebase_analytics';
  RAISE NOTICE '  ✓ api_keys';
  RAISE NOTICE '';
  RAISE NOTICE 'User data isolation is working correctly!';
  RAISE NOTICE '========================================';
END $$;

-- ============================================================================
-- CLEANUP (Optional - uncomment to remove test data)
-- ============================================================================

-- DELETE FROM api_keys WHERE user_id IN (1, 2);
-- DELETE FROM user_settings WHERE user_id IN (1, 2);
-- DELETE FROM dependencies WHERE project_id IN (1, 2);
-- DELETE FROM user_progress WHERE user_id IN (1, 2);
-- DELETE FROM learning_phases WHERE id IN (1, 2);
-- DELETE FROM timeline_events WHERE project_id IN (1, 2);
-- DELETE FROM tasks WHERE id IN (1, 2, 3);
-- DELETE FROM vulnerabilities WHERE project_id IN (1, 2);
-- DELETE FROM codebase_analytics WHERE project_id IN (1, 2);
-- DELETE FROM migrations WHERE project_id IN (1, 2);
-- DELETE FROM projects WHERE id IN (1, 2);
-- DELETE FROM users WHERE id IN (1, 2);
