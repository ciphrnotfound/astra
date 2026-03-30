// Database TypeScript Types for Astra Landing Application
// This file contains TypeScript interfaces for all database tables

// ============================================================================
// CLI-Synced Tables (Data from Rust CLI)
// ============================================================================

// Astra Sessions - synced from CLI teams.rs
export interface AstraSession {
  id: number;
  task_id: string;
  developer: string;
  start_time: number; // Unix timestamp
  end_time: number; // Unix timestamp
  lines_added: number;
  lines_deleted: number;
  prompts_asked: string[]; // JSONB array
  files_touched: string[]; // JSONB array
  created_at: string;
}

// Health Snapshots - synced from CLI health.rs
export interface HealthSnapshot {
  id: number;
  project_id: number;
  code_quality: number; // 0-100
  test_health: number; // 0-100
  cross_lang_drift: number; // 0-100
  security_surface: number; // 0-100
  git_health: number; // 0-100
  team_velocity: number; // 0-100
  total_lines: number;
  file_count: number;
  todo_count: number;
  test_files: number;
  language_count: number;
  migration_count: number;
  security_files: number;
  uncommitted_changes: number;
  recent_commits: number;
  tasks_done: number;
  tasks_total: number;
  timestamp: number; // Unix timestamp
  created_at: string;
}

// Health Scores (subset of HealthSnapshot)
export interface HealthScores {
  code_quality: number;
  test_health: number;
  cross_lang_drift: number;
  security_surface: number;
  git_health: number;
  team_velocity: number;
}

// Security Issues - synced from CLI security.rs
export interface SecurityIssue {
  id: number;
  project_id: number;
  severity: 'Critical' | 'High' | 'Medium' | 'Low';
  file_path: string;
  line_number: number;
  description: string;
  snippet: string;
  status: 'open' | 'resolved' | 'ignored';
  detected_at: number; // Unix timestamp
  resolved_at: number | null; // Unix timestamp
  created_at: string;
}

// Memory Event Data Types (from CLI memory.rs)
export type MemoryEventData =
  | { type: 'IndexSnapshot'; file_count: number; total_lines: number; languages: Record<string, number> }
  | { type: 'MigrationRun'; from: string; to: string; file_count: number }
  | { type: 'TeamSession'; developer: string; task_id: string; duration_secs: number; lines_added: number; lines_deleted: number }
  | { type: 'WorktreeSnapshot'; changed_files: number; files: string[] }
  | { type: 'HealthSnapshot'; scores: HealthScores }
  | { type: 'GitCommit'; id: string; summary: string; author: string; date: string }
  | { type: 'LearningProgress'; phase: LearningPhase };

// Timeline Events - synced from CLI memory.rs
export interface CLITimelineEvent {
  id: number;
  project_id: number;
  event_type: string;
  content: string;
  event_data: MemoryEventData | null; // JSONB
  timestamp: number; // Unix timestamp
  created_at: string;
}

// Tasks - synced from CLI teams.rs
export interface CLITask {
  id: number;
  project_id: number | null;
  task_id: string; // Unique task identifier from CLI
  description: string;
  assignee: string; // Developer name (not user_id)
  status: 'Pending' | 'InProgress' | 'Done';
  created_at: string;
  updated_at: string;
}

// ============================================================================
// Core Tables
// ============================================================================

export interface User {
  id: number;
  name: string | null;
  email: string;
  password_hash: string;
  role: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
}

export interface Team {
  id: number;
  name: string;
  created_at: string;
  updated_at: string;
  stripe_customer_id: string | null;
  stripe_subscription_id: string | null;
  stripe_product_id: string | null;
  plan_name: string | null;
  subscription_status: string | null;
}

export interface TeamMember {
  id: number;
  user_id: number;
  team_id: number;
  role: string;
  joined_at: string;
}

export interface ActivityLog {
  id: number;
  team_id: number;
  user_id: number | null;
  action: string;
  timestamp: string;
  ip_address: string | null;
}

export interface Invitation {
  id: number;
  team_id: number;
  email: string;
  role: string;
  invited_by: number;
  invited_at: string;
  status: string;
}

// ============================================================================
// Project & Migration Tables
// ============================================================================

export interface Project {
  id: number;
  user_id: number;
  name: string;
  description: string | null;
  repository_url: string | null;
  language: string | null;
  created_at: string;
  updated_at: string;
}

export interface Migration {
  id: number;
  project_id: number;
  user_id: number;
  source_language: string;
  target_language: string;
  status: string;
  files_processed: number;
  total_files: number;
  error_message: string | null;
  created_at: string;
  completed_at: string | null;
}

export interface CodebaseAnalytics {
  id: number;
  project_id: number;
  lines_of_code: number;
  files_count: number;
  technical_debt: number;
  test_coverage: number;
  security_score: number;
  analyzed_at: string;
}

export interface ApiKey {
  id: number;
  user_id: number;
  name: string;
  key_hash: string;
  key_prefix: string;
  last_used_at: string | null;
  created_at: string;
  expires_at: string | null;
  is_active: boolean;
}

// ============================================================================
// Security & Vulnerability Tables
// ============================================================================

export interface Vulnerability {
  id: number;
  project_id: number;
  severity: 'critical' | 'high' | 'medium' | 'low';
  title: string;
  description: string | null;
  file_path: string;
  line_number: number | null;
  cwe_id: string | null;
  remediation: string | null;
  status: 'open' | 'resolved' | 'ignored';
  detected_at: string;
  resolved_at: string | null;
  created_at: string;
}

// ============================================================================
// Task Management Tables
// ============================================================================

export interface Task {
  id: number;
  project_id: number | null;
  title: string;
  description: string | null;
  status: 'todo' | 'in_progress' | 'done';
  priority: 'low' | 'medium' | 'high' | 'urgent';
  assignee_id: number | null;
  created_by: number;
  tags: string[];
  due_date: string | null;
  completed_at: string | null;
  created_at: string;
  updated_at: string;
}

// ============================================================================
// Timeline & History Tables
// ============================================================================

export interface TimelineEvent {
  id: number;
  project_id: number;
  event_type: 'analysis' | 'migration' | 'refactor' | 'security_scan' | 'deployment';
  title: string;
  description: string | null;
  affected_files: string[];
  metadata: Record<string, any>;
  user_id: number | null;
  created_at: string;
}

// ============================================================================
// Learning & Onboarding Tables
// ============================================================================

export interface LearningPhase {
  id: number;
  title: string;
  description: string | null;
  content: string;
  order_index: number;
  estimated_minutes: number | null;
  prerequisites: number[];
  exercises: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface UserProgress {
  id: number;
  user_id: number;
  phase_id: number;
  status: 'not_started' | 'in_progress' | 'completed';
  progress_percentage: number;
  started_at: string | null;
  completed_at: string | null;
}

// ============================================================================
// Dependency & Graph Tables
// ============================================================================

export interface Dependency {
  id: number;
  project_id: number;
  source_file: string;
  target_file: string;
  dependency_type: string;
  line_number: number | null;
  metadata: Record<string, any>;
  created_at: string;
}

// ============================================================================
// Settings & Configuration Tables
// ============================================================================

export interface UserSettings {
  id: number;
  user_id: number;
  persona: {
    role: string;
    experience: string;
    preferences: Record<string, any>;
  };
  model_config: {
    model: string;
    temperature: number;
    max_tokens: number;
  };
  integrations: Record<string, any>;
  notifications: {
    email: boolean;
    realtime: boolean;
  };
  created_at: string;
  updated_at: string;
}

// ============================================================================
// Composite & Utility Types
// ============================================================================

export interface HealthMetrics {
  codeQuality: number;
  technicalDebt: number;
  testCoverage: number;
  securityScore: number;
  linesOfCode: number;
  filesCount: number;
}

export interface MetricTrend {
  date: string;
  value: number;
}

export interface DashboardStats {
  totalMigrations: number;
  filesProcessed: number;
  activeProjects: number;
  recentMigrations: Migration[];
}

// Aggregated Dashboard Stats (from CLI-synced data)
export interface CLIDashboardStats {
  totalSessions: number;
  totalLinesAdded: number;
  totalLinesDeleted: number;
  activeProjects: number;
  recentSessions: AstraSession[];
  latestHealthSnapshot: HealthSnapshot | null;
}

// Health Metrics with Trends
export interface HealthMetricsWithTrends {
  current: HealthScores;
  trend: HealthTrendPoint[];
  details: {
    total_lines: number;
    file_count: number;
    todo_count: number;
    test_files: number;
    language_count: number;
    migration_count: number;
    security_files: number;
  };
}

export interface HealthTrendPoint {
  timestamp: number;
  scores: HealthScores;
}

// ============================================================================
// Insert Types (for creating new records)
// ============================================================================

export type NewVulnerability = Omit<Vulnerability, 'id' | 'created_at'>;
export type NewTask = Omit<Task, 'id' | 'created_at' | 'updated_at'>;
export type NewTimelineEvent = Omit<TimelineEvent, 'id' | 'created_at'>;
export type NewLearningPhase = Omit<LearningPhase, 'id' | 'created_at' | 'updated_at'>;
export type NewUserProgress = Omit<UserProgress, 'id'>;
export type NewDependency = Omit<Dependency, 'id' | 'created_at'>;
export type NewUserSettings = Omit<UserSettings, 'id' | 'created_at' | 'updated_at'>;

// CLI-Synced Insert Types (used by Rust CLI for syncing)
export type NewAstraSession = Omit<AstraSession, 'id' | 'created_at'>;
export type NewHealthSnapshot = Omit<HealthSnapshot, 'id' | 'created_at'>;
export type NewSecurityIssue = Omit<SecurityIssue, 'id' | 'created_at'>;
export type NewCLITimelineEvent = Omit<CLITimelineEvent, 'id' | 'created_at'>;
export type NewCLITask = Omit<CLITask, 'id' | 'created_at' | 'updated_at'>;

// ============================================================================
// Update Types (for updating existing records)
// ============================================================================

export type UpdateVulnerability = Partial<Omit<Vulnerability, 'id' | 'created_at'>>;
export type UpdateTask = Partial<Omit<Task, 'id' | 'created_at'>>;
export type UpdateTimelineEvent = Partial<Omit<TimelineEvent, 'id' | 'created_at'>>;
export type UpdateLearningPhase = Partial<Omit<LearningPhase, 'id' | 'created_at'>>;
export type UpdateUserProgress = Partial<Omit<UserProgress, 'id' | 'user_id' | 'phase_id'>>;
export type UpdateUserSettings = Partial<Omit<UserSettings, 'id' | 'user_id' | 'created_at'>>;

// CLI-Synced Update Types (dashboard can update status fields)
export type UpdateSecurityIssue = Partial<Pick<SecurityIssue, 'status' | 'resolved_at'>>;
export type UpdateCLITask = Partial<Pick<CLITask, 'status' | 'updated_at'>>;

// ============================================================================
// Extended Types with Relations
// ============================================================================

export type VulnerabilityWithProject = Vulnerability & {
  project: Project;
};

export type TaskWithAssignee = Task & {
  assignee: Pick<User, 'id' | 'name' | 'email'> | null;
  creator: Pick<User, 'id' | 'name' | 'email'>;
};

export type TimelineEventWithUser = TimelineEvent & {
  user: Pick<User, 'id' | 'name' | 'email'> | null;
};

export type UserProgressWithPhase = UserProgress & {
  phase: LearningPhase;
};

export type ProjectWithStats = Project & {
  migrations_count: number;
  latest_analytics: CodebaseAnalytics | null;
  vulnerabilities_count: number;
};

// ============================================================================
// Filter & Query Types
// ============================================================================

export interface VulnerabilityFilters {
  severity?: Vulnerability['severity'][];
  status?: Vulnerability['status'][];
  project_id?: number;
}

export interface TaskFilters {
  status?: Task['status'][];
  priority?: Task['priority'][];
  assignee_id?: number;
  project_id?: number;
  tags?: string[];
}

export interface TimelineEventFilters {
  event_type?: TimelineEvent['event_type'][];
  project_id?: number;
  date_from?: string;
  date_to?: string;
}

// CLI-Synced Data Filters
export interface SecurityIssueFilters {
  severity?: SecurityIssue['severity'][];
  status?: SecurityIssue['status'][];
  project_id?: number;
}

export interface CLITaskFilters {
  status?: CLITask['status'][];
  assignee?: string;
  project_id?: number;
}

export interface CLITimelineEventFilters {
  event_type?: string[];
  project_id?: number;
  since?: number; // Unix timestamp
  until?: number; // Unix timestamp
}

export interface AstraSessionFilters {
  developer?: string;
  task_id?: string;
  project_id?: number;
  since?: number; // Unix timestamp
  until?: number; // Unix timestamp
}

export interface HealthSnapshotFilters {
  project_id?: number;
  since?: number; // Unix timestamp
  until?: number; // Unix timestamp
  time_range?: '7d' | '30d' | '90d';
}

export interface PaginationParams {
  page: number;
  limit: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}
