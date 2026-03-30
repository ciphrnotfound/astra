# Design Document: Content and Backend Integration

## Overview

This design document specifies the technical architecture for completing the Astra landing page and dashboard with real content and full backend integration. Astra is a "Codebase OS" that helps developers understand and manage their codebases through semantic analysis, health tracking, and migration management.

### System Architecture Overview

Astra uses a **3-tier architecture**:

1. **Rust CLI** (`core/` directory) - Runs locally on developer machines
   - Tracks team productivity, migrations, health metrics, security scans
   - Has modules: teams.rs, health.rs, security.rs, memory.rs, migration.rs, etc.
   - Stores data locally in `~/.astra/` directory
   - Syncs data to Supabase when online via `supabase.rs` module

2. **Supabase Cloud Database** - Central data store
   - Receives data from Rust CLI via sync operations
   - Stores: team sessions, tasks, migrations, health metrics, security vulnerabilities, timeline events
   - The Rust CLI already has code to sync to `astra_sessions` table

3. **Next.js Dashboard** (`astra-landing-next/`) - Web interface
   - Marketing landing page + authenticated dashboard
   - **Reads data from Supabase** (does NOT generate data itself)
   - Displays team productivity, migration history, health metrics, etc.
   - Users sign in with GitHub OAuth or email/password

**Data Flow:**
```
Developer Machine → Rust CLI → Supabase ← Next.js Dashboard (read-only)
```

### Current State

The application currently has:
- Basic Next.js 14 App Router structure with TypeScript
- Supabase backend with partial schema (users, projects, migrations, codebase_analytics)
- Placeholder dashboard pages with static content
- Authentication scaffolding without full session management
- Design system established (Cabinet Grotesk font, square borders, minimal animations)
- Rust CLI that generates and syncs data to Supabase `astra_sessions` table

### Target State

The completed system will provide:
- Comprehensive documentation system with searchable navigation
- Complete footer pages (privacy, terms, blog, careers, contact, integrations, pricing)
- Fully functional dashboard displaying real data synced from Rust CLI
- Six health metrics with historical trend visualization (data from CLI health.rs module)
- Interactive dependency graph (Semantic Cartographer) - data from CLI index
- Security vulnerability tracking (Security Hunter) - data from CLI security.rs module
- Task management with team velocity metrics (data from CLI teams.rs module)
- Codebase memory timeline (data from CLI memory.rs module)
- Onboarding learning system with progress tracking
- Settings management for persona, models, team, and integrations
- Complete authentication flow with session management
- Real-time updates using Supabase subscriptions
- Performance optimizations (pagination, lazy loading, caching)
- Database schema aligned with Rust CLI data structures

### Technology Stack

- **CLI**: Rust (data generation and sync)
- **Frontend**: Next.js 14 (App Router), React 18, TypeScript
- **Styling**: Tailwind CSS, Framer Motion for animations
- **Backend**: Supabase (PostgreSQL + Realtime + Auth)
- **State Management**: React Server Components + Client Components with hooks
- **Data Fetching**: Server Actions, Supabase client/server utilities
- **Visualization**: D3.js or React Flow for dependency graphs
- **Markdown**: next-mdx-remote or similar for documentation/blog content


## Architecture

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Developer Machine (Local)                   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  Rust CLI (Astra)                     │   │
│  │  • teams.rs - Track sessions, tasks, velocity        │   │
│  │  • health.rs - Compute 6 health metrics              │   │
│  │  • security.rs - Scan for vulnerabilities            │   │
│  │  • memory.rs - Record timeline events                │   │
│  │  • migration.rs - Execute code migrations            │   │
│  │  • index.rs - Build dependency graph                 │   │
│  │  • supabase.rs - Sync data to cloud                  │   │
│  └──────────────────────────────────────────────────────┘   │
│                            │                                  │
│                            │ Stores locally in ~/.astra/      │
│                            │ Syncs to Supabase when online    │
└────────────────────────────┼─────────────────────────────────┘
                             │
                             │ HTTP POST to Supabase REST API
                             │
┌────────────────────────────▼─────────────────────────────────┐
│                    Supabase Backend (Cloud)                  │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              PostgreSQL Database                      │   │
│  │  • astra_sessions (synced from CLI)                  │   │
│  │  • health_snapshots (synced from CLI)                │   │
│  │  • security_issues (synced from CLI)                 │   │
│  │  • timeline_events (synced from CLI)                 │   │
│  │  • dependencies (synced from CLI)                    │   │
│  │  • users, projects (dashboard managed)               │   │
│  │  • learning_phases, user_progress (dashboard)        │   │
│  │  • user_settings (dashboard managed)                 │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Realtime Subscriptions                   │   │
│  │  • Session updates from CLI                           │   │
│  │  • Health metric changes                              │   │
│  │  • Security scan results                              │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Authentication                           │   │
│  │  • GitHub OAuth, Email/Password                       │   │
│  │  • Row Level Security (RLS)                           │   │
│  └──────────────────────────────────────────────────────┘   │
│                            ▲                                  │
└────────────────────────────┼─────────────────────────────────┘
                             │
                             │ Supabase Client SDK (read-only)
                             │
┌────────────────────────────┼─────────────────────────────────┐
│                  Next.js Dashboard (Web)                     │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Landing    │  │     Docs     │  │    Footer    │      │
│  │    Pages     │  │    System    │  │    Pages     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │         Dashboard (Read-Only Visualization)          │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐│    │
│  │  │ Overview │ │  Health  │ │  Graph   │ │Security││    │
│  │  │ (sessions│ │(CLI data)│ │(CLI deps)│ │(CLI    ││    │
│  │  │ from CLI)│ │          │ │          │ │scans)  ││    │
│  │  └──────────┘ └──────────┘ └──────────┘ └────────┘│    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐│    │
│  │  │  Tasks   │ │  Memory  │ │Onboarding│ │Settings││    │
│  │  │(CLI data)│ │(CLI data)│ │(dashboard│ │(write) ││    │
│  │  │          │ │          │ │managed)  │ │        ││    │
│  │  └──────────┘ └──────────┘ └──────────┘ └────────┘│    │
│  └─────────────────────────────────────────────────────┘    │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**Key Architectural Points:**

1. **Data Generation**: The Rust CLI is the primary data generator. It runs on developer machines and tracks all codebase activity.

2. **Data Sync**: The CLI syncs data to Supabase via REST API (see `core/src/supabase.rs`). Currently syncs to `astra_sessions` table.

3. **Dashboard Role**: The Next.js dashboard is primarily a **read-only visualization layer**. It displays data synced from the CLI.

4. **Write Operations**: Some dashboard features require write capabilities:
   - User settings and preferences
   - Onboarding progress tracking
   - Team member invitations (admin actions)
   - Project metadata management

5. **Schema Alignment**: The Supabase schema must match the data structures in the Rust CLI (Session, HealthScores, SecurityIssue, MemoryEvent, etc.)

### Application Structure

```
astra-landing-next/
├── app/
│   ├── (marketing)/          # Public pages
│   │   ├── page.tsx          # Landing page
│   │   ├── about/
│   │   ├── privacy/
│   │   ├── terms/
│   │   ├── blog/
│   │   ├── careers/
│   │   ├── contact/
│   │   ├── integrations/
│   │   └── pricing/
│   ├── docs/                 # Documentation system
│   │   ├── layout.tsx
│   │   ├── page.tsx
│   │   └── [...slug]/
│   ├── dashboard/            # Authenticated dashboard
│   │   ├── layout.tsx
│   │   ├── page.tsx          # Overview
│   │   ├── health/
│   │   ├── graph/
│   │   ├── migrations/
│   │   ├── security/
│   │   ├── tasks/
│   │   ├── memory/
│   │   ├── onboarding/
│   │   ├── settings/
│   │   └── projects/
│   ├── api/                  # API routes
│   │   ├── auth/
│   │   ├── dashboard/
│   │   ├── migrations/
│   │   └── contact/
│   └── signin/
├── components/
│   ├── dashboard/            # Dashboard-specific components
│   ├── docs/                 # Documentation components
│   ├── marketing/            # Landing page components
│   └── ui/                   # Shared UI components
├── lib/
│   ├── supabase/
│   │   ├── client.ts         # Client-side Supabase
│   │   ├── server.ts         # Server-side Supabase
│   │   └── middleware.ts     # Auth middleware
│   ├── db/
│   │   ├── schema.ts         # Database schema types
│   │   └── queries.ts        # Reusable queries
│   ├── hooks/                # Custom React hooks
│   └── utils/                # Utility functions
└── content/
    ├── docs/                 # Markdown documentation
    └── blog/                 # Blog posts
```

### Data Flow Patterns

#### Read-Only Dashboard Features (CLI Data)
These features display data synced from the Rust CLI. The dashboard only reads this data:

- **Overview Page**: Display sessions, lines added/deleted, active projects
- **Health Metrics**: Display 6 health scores and trends from health_snapshots
- **Security Hunter**: Display vulnerabilities from security_issues
- **Semantic Cartographer**: Display dependency graph from dependencies table
- **Codebase Memory**: Display timeline events from timeline_events
- **Tasks (Read)**: Display tasks synced from CLI teams.rs

#### Write-Enabled Dashboard Features
These features allow user interaction and write to the database:

- **Settings**: User preferences, persona, model config, integrations
- **Onboarding**: Learning progress tracking
- **Project Management**: Create/edit/delete projects (metadata only)
- **Security Issues (Update)**: Mark vulnerabilities as resolved/ignored
- **Tasks (Update)**: Change task status (admin actions)

#### Server Component Data Fetching (Read-Only)
```typescript
// Server Component (default in App Router)
async function DashboardPage() {
  const supabase = createServerClient();
  
  // Fetch CLI-synced data (read-only)
  const { data: sessions } = await supabase
    .from('astra_sessions')
    .select('*')
    .order('start_time', { ascending: false })
    .limit(10);
  
  const { data: latestHealth } = await supabase
    .from('health_snapshots')
    .select('*')
    .order('timestamp', { ascending: false })
    .limit(1)
    .single();
  
  return <DashboardView sessions={sessions} health={latestHealth} />;
}
```

#### Client Component with Real-time Updates
```typescript
// Client Component for real-time CLI data updates
'use client';
function HealthMetricsLive({ initialHealth }) {
  const [health, setHealth] = useState(initialHealth);
  
  useEffect(() => {
    const channel = supabase
      .channel('health_updates')
      .on('postgres_changes', 
        { event: 'INSERT', schema: 'public', table: 'health_snapshots' },
        (payload) => setHealth(payload.new)
      )
      .subscribe();
    
    return () => supabase.removeChannel(channel);
  }, []);
  
  return <HealthDisplay health={health} />;
}
```

#### Optimistic Updates Pattern (Write Operations)
```typescript
// For dashboard-managed data (user settings, progress, etc.)
async function updateUserSettings(settings: Partial<UserSettings>) {
  // Optimistic update
  setSettings(prev => ({ ...prev, ...settings }));
  
  // Actual update
  const { error } = await supabase
    .from('user_settings')
    .update(settings)
    .eq('user_id', userId);
  
  if (error) {
    // Revert on error
    setSettings(initialSettings);
    showError(error.message);
  }
}
```


## Components and Interfaces

### Core Component Architecture

#### Dashboard Layout Component
```typescript
// app/dashboard/layout.tsx
interface DashboardLayoutProps {
  children: React.ReactNode;
}

// Provides:
// - Authentication check
// - Navigation sidebar
// - User context
// - Real-time connection status
```

#### Data Fetching Hooks
```typescript
// lib/hooks/useDashboardStats.ts
interface DashboardStats {
  totalSessions: number;
  totalLinesAdded: number;
  totalLinesDeleted: number;
  activeProjects: number;
  recentSessions: AstraSession[];
  latestHealth: HealthSnapshot | null;
}

function useDashboardStats(): {
  stats: DashboardStats | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

// lib/hooks/useHealthMetrics.ts
interface HealthMetrics {
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

function useHealthMetrics(
  projectId: number,
  timeRange: '7d' | '30d' | '90d'
): {
  metrics: HealthMetrics | null;
  loading: boolean;
  error: Error | null;
}

// lib/hooks/useSecurityIssues.ts
function useSecurityIssues(
  projectId: number,
  filters?: { severity?: string; status?: string }
): {
  issues: SecurityIssue[];
  loading: boolean;
  error: Error | null;
  updateStatus: (issueId: number, status: string) => Promise<void>;
}

// lib/hooks/useTimelineEvents.ts
function useTimelineEvents(
  projectId: number,
  filters?: { eventType?: string; since?: number }
): {
  events: TimelineEvent[];
  loading: boolean;
  error: Error | null;
}

// lib/hooks/useDependencyGraph.ts
function useDependencyGraph(projectId: number): {
  dependencies: Dependency[];
  loading: boolean;
  error: Error | null;
}

// lib/hooks/useRealtime.ts
function useRealtimeSubscription<T>(
  table: string,
  filter?: { column: string; value: any }
): {
  data: T[];
  loading: boolean;
  error: Error | null;
}
```

#### Component Library Structure

**Dashboard Components:**
- `<StatCard />` - Metric display with trend indicator
- `<EmptyState />` - Placeholder for empty data
- `<LoadingState />` - Skeleton screens
- `<ErrorBoundary />` - Error handling wrapper
- `<DataTable />` - Paginated table with sorting/filtering
- `<MetricChart />` - Line/bar charts for trends
- `<DependencyGraph />` - Interactive node-link diagram
- `<TaskCard />` - Draggable task item
- `<TimelineEvent />` - Event display in timeline
- `<LearningPhaseCard />` - Onboarding phase display

**Form Components:**
- `<ProjectForm />` - Create/edit projects
- `<MigrationForm />` - Configure migrations
- `<TaskForm />` - Create/edit tasks
- `<SettingsForm />` - User preferences
- `<ContactForm />` - Contact page form

**Documentation Components:**
- `<DocsLayout />` - Documentation page wrapper
- `<DocsSidebar />` - Navigation tree
- `<DocsContent />` - Markdown renderer with syntax highlighting
- `<DocsSearch />` - Search functionality
- `<CodeBlock />` - Syntax-highlighted code examples

### API Route Structure

```typescript
// app/api/dashboard/stats/route.ts
export async function GET(request: Request) {
  const supabase = createServerClient();
  const { data: { user } } = await supabase.auth.getUser();
  
  if (!user) {
    return Response.json({ error: 'Unauthorized' }, { status: 401 });
  }
  
  // Fetch stats with RLS automatically applied
  const stats = await getDashboardStats(user.id);
  
  return Response.json(stats);
}

// app/api/migrations/route.ts
export async function POST(request: Request) {
  const body = await request.json();
  const validation = migrationSchema.safeParse(body);
  
  if (!validation.success) {
    return Response.json(
      { error: validation.error },
      { status: 400 }
    );
  }
  
  const migration = await createMigration(validation.data);
  return Response.json(migration, { status: 201 });
}

// app/api/contact/route.ts
export async function POST(request: Request) {
  const { name, email, message } = await request.json();
  
  // Validate
  if (!email || !message) {
    return Response.json(
      { error: 'Email and message required' },
      { status: 400 }
    );
  }
  
  // Send email via service (e.g., Resend, SendGrid)
  await sendContactEmail({ name, email, message });
  
  return Response.json({ success: true });
}
```

### Authentication Flow

```typescript
// lib/supabase/middleware.ts
export async function requireAuth(request: Request) {
  const supabase = createServerClient();
  const { data: { session } } = await supabase.auth.getSession();
  
  if (!session) {
    return redirect('/signin');
  }
  
  return session.user;
}

// Usage in Server Component
async function ProtectedPage() {
  const user = await requireAuth(request);
  // Page content
}
```


## Data Models

### Schema Design Philosophy

The Supabase database schema must align with the Rust CLI data structures to ensure seamless data sync. The CLI generates data locally and syncs it to Supabase, so the database tables must match the Rust struct definitions.

**Data Sources:**
- **CLI-Generated**: Sessions, health metrics, security issues, timeline events, dependencies
- **Dashboard-Managed**: User settings, learning progress, project metadata, team invitations

### Extended Database Schema

#### CLI-Synced Tables

**astra_sessions** (already exists, synced from CLI)
```sql
CREATE TABLE astra_sessions (
  id SERIAL PRIMARY KEY,
  task_id VARCHAR(255) NOT NULL,
  developer VARCHAR(255) NOT NULL,
  start_time BIGINT NOT NULL,
  end_time BIGINT NOT NULL,
  lines_added INTEGER NOT NULL DEFAULT 0,
  lines_deleted INTEGER NOT NULL DEFAULT 0,
  prompts_asked JSONB DEFAULT '[]',
  files_touched JSONB DEFAULT '[]',
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_astra_sessions_developer ON astra_sessions(developer);
CREATE INDEX idx_astra_sessions_task_id ON astra_sessions(task_id);
CREATE INDEX idx_astra_sessions_start_time ON astra_sessions(start_time DESC);
```

**health_snapshots** (synced from CLI health.rs)
```sql
CREATE TABLE health_snapshots (
  id SERIAL PRIMARY KEY,
  project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
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

CREATE INDEX idx_health_snapshots_project_id ON health_snapshots(project_id);
CREATE INDEX idx_health_snapshots_timestamp ON health_snapshots(timestamp DESC);
```

**security_issues** (synced from CLI security.rs)
```sql
CREATE TABLE security_issues (
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

CREATE INDEX idx_security_issues_project_id ON security_issues(project_id);
CREATE INDEX idx_security_issues_severity ON security_issues(severity);
CREATE INDEX idx_security_issues_status ON security_issues(status);
```

**timeline_events** (synced from CLI memory.rs)
```sql
CREATE TABLE timeline_events (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  event_type VARCHAR(50) NOT NULL,
  content TEXT NOT NULL,
  event_data JSONB,
  timestamp BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_timeline_events_project_id ON timeline_events(project_id);
CREATE INDEX idx_timeline_events_type ON timeline_events(event_type);
CREATE INDEX idx_timeline_events_timestamp ON timeline_events(timestamp DESC);

-- event_data JSONB structure varies by event_type:
-- IndexSnapshot: { file_count, total_lines, languages: {...} }
-- MigrationRun: { from, to, file_count }
-- TeamSession: { developer, task_id, duration_secs, lines_added, lines_deleted }
-- WorktreeSnapshot: { changed_files, files: [...] }
-- HealthSnapshot: { scores: {...} }
-- GitCommit: { id, summary, author, date }
-- LearningProgress: { phase: {...} }
```

**dependencies** (synced from CLI index.rs)
```sql
CREATE TABLE dependencies (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  source_file TEXT NOT NULL,
  target_file TEXT NOT NULL,
  dependency_type VARCHAR(50) NOT NULL,
  metadata JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, source_file, target_file, dependency_type)
);

CREATE INDEX idx_dependencies_project_id ON dependencies(project_id);
CREATE INDEX idx_dependencies_source_file ON dependencies(source_file);
CREATE INDEX idx_dependencies_target_file ON dependencies(target_file);
```

**tasks** (synced from CLI teams.rs)
```sql
CREATE TABLE tasks (
  id SERIAL PRIMARY KEY,
  project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
  task_id VARCHAR(255) NOT NULL UNIQUE,
  description TEXT NOT NULL,
  assignee VARCHAR(255) NOT NULL,
  status VARCHAR(20) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'InProgress', 'Done')),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_assignee ON tasks(assignee);
CREATE INDEX idx_tasks_status ON tasks(status);
```

#### Dashboard-Managed Tables

**learning_phases**
```sql
CREATE TABLE learning_phases (
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

CREATE INDEX idx_learning_phases_order ON learning_phases(order_index);
```

**user_progress**
```sql
CREATE TABLE user_progress (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  phase_id INTEGER NOT NULL REFERENCES learning_phases(id) ON DELETE CASCADE,
  status VARCHAR(20) NOT NULL DEFAULT 'not_started' CHECK (status IN ('not_started', 'in_progress', 'completed')),
  progress_percentage INTEGER DEFAULT 0 CHECK (progress_percentage >= 0 AND progress_percentage <= 100),
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  UNIQUE(user_id, phase_id)
);

CREATE INDEX idx_user_progress_user_id ON user_progress(user_id);
CREATE INDEX idx_user_progress_phase_id ON user_progress(phase_id);
```

**user_settings**
```sql
CREATE TABLE user_settings (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
  persona JSONB DEFAULT '{"role": "developer", "experience": "intermediate", "preferences": {}}',
  model_config JSONB DEFAULT '{"model": "gpt-4", "temperature": 0.7, "max_tokens": 2000}',
  integrations JSONB DEFAULT '{}',
  notifications JSONB DEFAULT '{"email": true, "realtime": true}',
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
```

### TypeScript Interfaces

```typescript
// lib/db/types.ts

// CLI-Synced Types (read-only from dashboard perspective)

export interface AstraSession {
  id: number;
  task_id: string;
  developer: string;
  start_time: number; // Unix timestamp
  end_time: number; // Unix timestamp
  lines_added: number;
  lines_deleted: number;
  prompts_asked: string[];
  files_touched: string[];
  created_at: string;
}

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

export interface TimelineEvent {
  id: number;
  project_id: number;
  event_type: string;
  content: string;
  event_data: MemoryEventData | null;
  timestamp: number; // Unix timestamp
  created_at: string;
}

export type MemoryEventData =
  | { type: 'IndexSnapshot'; file_count: number; total_lines: number; languages: Record<string, number> }
  | { type: 'MigrationRun'; from: string; to: string; file_count: number }
  | { type: 'TeamSession'; developer: string; task_id: string; duration_secs: number; lines_added: number; lines_deleted: number }
  | { type: 'WorktreeSnapshot'; changed_files: number; files: string[] }
  | { type: 'HealthSnapshot'; scores: HealthScores }
  | { type: 'GitCommit'; id: string; summary: string; author: string; date: string }
  | { type: 'LearningProgress'; phase: LearningPhase };

export interface HealthScores {
  code_quality: number;
  test_health: number;
  cross_lang_drift: number;
  security_surface: number;
  git_health: number;
  team_velocity: number;
}

export interface Dependency {
  id: number;
  project_id: number;
  source_file: string;
  target_file: string;
  dependency_type: string;
  metadata: Record<string, any> | null;
  created_at: string;
}

export interface Task {
  id: number;
  project_id: number | null;
  task_id: string;
  description: string;
  assignee: string;
  status: 'Pending' | 'InProgress' | 'Done';
  created_at: string;
  updated_at: string;
}

// Dashboard-Managed Types (read-write)

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

// Aggregated Dashboard Types

export interface DashboardStats {
  totalSessions: number;
  totalLinesAdded: number;
  totalLinesDeleted: number;
  activeProjects: number;
  recentSessions: AstraSession[];
  latestHealthSnapshot: HealthSnapshot | null;
}

export interface HealthMetrics {
  current: HealthScores;
  trend: HealthTrendPoint[];
}

export interface HealthTrendPoint {
  timestamp: number;
  scores: HealthScores;
}
```

### Row Level Security (RLS) Policies

```sql
-- Enable RLS on all tables
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE astra_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE health_snapshots ENABLE ROW LEVEL SECURITY;
ALTER TABLE security_issues ENABLE ROW LEVEL SECURITY;
ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE timeline_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE dependencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_progress ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_settings ENABLE ROW LEVEL SECURITY;

-- Projects: Users can only see their own projects
CREATE POLICY "Users can view own projects"
  ON projects FOR SELECT
  USING (auth.uid()::integer = user_id);

CREATE POLICY "Users can insert own projects"
  ON projects FOR INSERT
  WITH CHECK (auth.uid()::integer = user_id);

CREATE POLICY "Users can update own projects"
  ON projects FOR UPDATE
  USING (auth.uid()::integer = user_id);

-- Astra Sessions: Users can view sessions for their projects
-- Note: CLI syncs sessions with developer name, not user_id
-- Dashboard needs to map developer names to users or use project_id
CREATE POLICY "Users can view project sessions"
  ON astra_sessions FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = astra_sessions.project_id
      AND projects.user_id = auth.uid()::integer
    )
  );

-- Health Snapshots: Users can view snapshots for their projects
CREATE POLICY "Users can view project health"
  ON health_snapshots FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = health_snapshots.project_id
      AND projects.user_id = auth.uid()::integer
    )
  );

-- Security Issues: Users can view and update issues for their projects
CREATE POLICY "Users can view project security issues"
  ON security_issues FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = security_issues.project_id
      AND projects.user_id = auth.uid()::integer
    )
  );

CREATE POLICY "Users can update project security issues"
  ON security_issues FOR UPDATE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = security_issues.project_id
      AND projects.user_id = auth.uid()::integer
    )
  );

-- Tasks: Users can view and manage tasks for their projects
CREATE POLICY "Users can view project tasks"
  ON tasks FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = tasks.project_id
      AND projects.user_id = auth.uid()::integer
    )
  );

CREATE POLICY "Users can update project tasks"
  ON tasks FOR UPDATE
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = tasks.project_id
      AND projects.user_id = auth.uid()::integer
    )
  );

-- Timeline Events: Users can view events for their projects
CREATE POLICY "Users can view project timeline"
  ON timeline_events FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = timeline_events.project_id
      AND projects.user_id = auth.uid()::integer
    )
  );

-- Dependencies: Users can view dependencies for their projects
CREATE POLICY "Users can view project dependencies"
  ON dependencies FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM projects
      WHERE projects.id = dependencies.project_id
      AND projects.user_id = auth.uid()::integer
    )
  );

-- User Progress: Users can only access their own progress
CREATE POLICY "Users can manage own progress"
  ON user_progress FOR ALL
  USING (auth.uid()::integer = user_id);

-- User Settings: Users can only access their own settings
CREATE POLICY "Users can manage own settings"
  ON user_settings FOR ALL
  USING (auth.uid()::integer = user_id);
```

### CLI Sync Considerations

**Authentication for CLI Sync:**
The Rust CLI uses API key authentication (not user sessions) to sync data. The CLI sync endpoint needs to:

1. Accept API key in Authorization header (configured via `astra config set supabase_key`)
2. Map API key to project_id (or use service role key)
3. Allow INSERT operations on CLI-synced tables
4. Validate that the API key has permission to write to the project

**Service Role for CLI:**
```sql
-- Create service role policies that allow CLI to insert data
-- These bypass RLS when using the service role key
-- The CLI must include the project_id in the payload

CREATE POLICY "Service role can insert sessions"
  ON astra_sessions FOR INSERT
  WITH CHECK (true);

CREATE POLICY "Service role can insert health snapshots"
  ON health_snapshots FOR INSERT
  WITH CHECK (true);

CREATE POLICY "Service role can insert security issues"
  ON security_issues FOR INSERT
  WITH CHECK (true);

CREATE POLICY "Service role can insert timeline events"
  ON timeline_events FOR INSERT
  WITH CHECK (true);

CREATE POLICY "Service role can insert dependencies"
  ON dependencies FOR INSERT
  WITH CHECK (true);

CREATE POLICY "Service role can insert tasks"
  ON tasks FOR INSERT
  WITH CHECK (true);
```

**Note:** These service role policies should only be active when using the service role key (configured in CLI). Regular user authentication uses the user-scoped policies above.

**CLI Schema Updates Needed:**
The current CLI `supabase.rs` only syncs to `astra_sessions`. It needs to be extended to sync:
- Health snapshots from `health.rs`
- Security issues from `security.rs`
- Timeline events from `memory.rs`
- Dependencies from `index.rs`
- Tasks from `teams.rs`

Each sync operation should include the `project_id` to associate data with the correct project.



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing the acceptance criteria, several properties can be combined or are redundant:

- Properties 4.2-4.7 (displaying individual health metrics) can be combined into one property that verifies all 6 metrics are displayed
- Properties 3.1-3.3 (fetching different counts) can be combined into one property about dashboard stats aggregation
- Properties 13.1, 13.3, 13.4 (authentication checks) can be combined into one property about auth enforcement

### Property 1: Documentation Search Returns Relevant Results

*For any* search query in the documentation system, all returned results should contain the search terms or semantically related content.

**Validates: Requirements 1.9**

### Property 2: Contact Form Validation Rejects Invalid Input

*For any* contact form submission with missing required fields (email or message), the system should reject the submission and display field-specific error messages.

**Validates: Requirements 2.6**

### Property 3: Dashboard Stats Aggregation

*For any* authenticated user, fetching dashboard stats should return aggregated counts (sessions, lines added/deleted, active projects) that match the sum of the user's data in the database.

**Validates: Requirements 3.1, 3.2, 3.3**

### Property 4: Recent Sessions Limit

*For any* authenticated user, fetching recent sessions should return at most the requested limit (e.g., 5 or 10) and should be ordered by start_time descending.

**Validates: Requirements 3.4**

### Property 5: Data Fetch Error Handling

*For any* data fetch operation that fails, the system should display an error message and provide a retry mechanism without losing application state.

**Validates: Requirements 3.9**

### Property 6: Real-time Data Updates

*For any* real-time subscription to CLI-synced tables (sessions, health_snapshots, security_issues), when new data is inserted, the dashboard should update the displayed data without page refresh.

**Validates: Requirements 3.10**

### Property 7: Health Metrics Completeness

*For any* health snapshot fetched from the database, the system should display all six health scores (code_quality, test_health, cross_lang_drift, security_surface, git_health, team_velocity) with their current values.

**Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7**

### Property 8: Health Trend Data Fetching

*For any* time range selection (7d, 30d, 90d), the system should fetch health snapshots within that time range and order them by timestamp ascending for trend visualization.

**Validates: Requirements 4.8, 4.10**

### Property 9: Authentication Enforcement

*For any* dashboard page request without a valid session, the system should redirect to the sign-in page and prevent access to protected data.

**Validates: Requirements 13.1, 13.3, 13.4**

### Property 10: Session Cookie Management

*For any* successful authentication, the system should store the session in secure, HTTP-only cookies, and for any sign-out action, the system should clear the session cookies.

**Validates: Requirements 13.2, 13.6**

### Property 11: User Data Isolation

*For any* database query from the dashboard, the system should include the authenticated user's ID in the query filter (via RLS) to ensure users can only access their own data.

**Validates: Requirements 13.5**

### Property 12: Session Refresh

*For any* valid but expiring session, the system should automatically refresh the session without requiring the user to re-authenticate.

**Validates: Requirements 13.10**

### Property 13: Foreign Key Constraint Enforcement

*For any* attempt to insert data with an invalid foreign key reference (e.g., project_id that doesn't exist), the database should reject the operation and return an error.

**Validates: Requirements 18.9**


## Error Handling

### Error Categories

**1. Authentication Errors**
- Invalid credentials
- Expired sessions
- Missing authentication tokens
- Insufficient permissions

**2. Data Fetch Errors**
- Network failures
- Database connection issues
- Query timeouts
- Invalid query parameters

**3. Data Sync Errors (CLI to Supabase)**
- API key authentication failures
- Invalid payload format
- Missing required fields (project_id)
- Rate limiting

**4. Validation Errors**
- Missing required form fields
- Invalid data formats
- Constraint violations

**5. Real-time Subscription Errors**
- Connection failures
- Subscription timeout
- Channel errors

### Error Handling Strategies

**Client-Side Error Handling:**
```typescript
// Graceful degradation for data fetch errors
async function fetchDashboardStats() {
  try {
    const { data, error } = await supabase
      .from('astra_sessions')
      .select('*');
    
    if (error) throw error;
    return data;
  } catch (error) {
    console.error('Failed to fetch dashboard stats:', error);
    // Show error UI with retry option
    showErrorToast('Failed to load dashboard data', {
      action: 'Retry',
      onAction: () => fetchDashboardStats()
    });
    return null;
  }
}

// Real-time subscription error handling
const channel = supabase
  .channel('health_updates')
  .on('postgres_changes', { ... }, handleUpdate)
  .subscribe((status, error) => {
    if (status === 'CHANNEL_ERROR') {
      console.error('Subscription error:', error);
      // Attempt reconnection
      setTimeout(() => channel.subscribe(), 5000);
    }
  });
```

**Server-Side Error Handling:**
```typescript
// API route error handling
export async function GET(request: Request) {
  try {
    const supabase = createServerClient();
    const { data: { user } } = await supabase.auth.getUser();
    
    if (!user) {
      return Response.json(
        { error: 'Unauthorized' },
        { status: 401 }
      );
    }
    
    const stats = await getDashboardStats(user.id);
    return Response.json(stats);
  } catch (error) {
    console.error('API error:', error);
    return Response.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
```

**CLI Sync Error Handling:**
The Rust CLI should handle sync errors gracefully:
- Queue failed sync operations for retry
- Log sync errors to local file
- Provide clear error messages to users
- Continue operating offline if sync fails

### Error Logging

**Dashboard Errors:**
- Log to browser console for development
- Send to error tracking service (e.g., Sentry) in production
- Include user context and error stack traces

**CLI Sync Errors:**
- Log to `~/.astra/sync_errors.log`
- Include timestamp, error type, and payload
- Provide CLI command to view sync status


## Testing Strategy

### Dual Testing Approach

The testing strategy uses both unit tests and property-based tests:

**Unit Tests:**
- Specific examples and edge cases
- Integration points between components
- Error conditions and boundary cases
- UI component rendering

**Property-Based Tests:**
- Universal properties across all inputs
- Comprehensive input coverage through randomization
- Data integrity and consistency
- Authentication and authorization rules

### Property-Based Testing Configuration

**Library Selection:**
- **TypeScript/JavaScript**: Use `fast-check` for property-based testing
- **Rust CLI**: Use `proptest` or `quickcheck` for property-based testing

**Test Configuration:**
- Minimum 100 iterations per property test
- Each property test must reference its design document property
- Tag format: `Feature: content-and-backend-integration, Property {number}: {property_text}`

### Test Categories

**1. Data Fetching Tests**

Unit Tests:
- Test dashboard stats fetch with mock data
- Test empty state handling
- Test error state handling
- Test loading state transitions

Property Tests:
- Property 3: Dashboard stats aggregation matches database sums
- Property 4: Recent sessions limit and ordering
- Property 8: Health trend data fetching for different time ranges

**2. Authentication Tests**

Unit Tests:
- Test sign-in flow with valid credentials
- Test sign-in flow with invalid credentials
- Test sign-out flow
- Test session cookie creation

Property Tests:
- Property 9: Authentication enforcement on all protected routes
- Property 10: Session cookie management (create on sign-in, clear on sign-out)
- Property 11: User data isolation via RLS
- Property 12: Session refresh without re-authentication

**3. Real-time Subscription Tests**

Unit Tests:
- Test subscription setup and teardown
- Test subscription error handling
- Test reconnection logic

Property Tests:
- Property 6: Real-time updates for CLI-synced data

**4. Form Validation Tests**

Unit Tests:
- Test contact form with valid input
- Test contact form with missing email
- Test contact form with missing message
- Test contact form success message

Property Tests:
- Property 2: Contact form validation rejects invalid input

**5. Database Schema Tests**

Unit Tests:
- Test table existence (Requirements 18.1-18.8)
- Test index existence (Requirement 18.10)

Property Tests:
- Property 13: Foreign key constraint enforcement

**6. CLI Sync Tests (Rust)**

Unit Tests:
- Test session sync payload format
- Test health snapshot sync payload format
- Test security issue sync payload format
- Test sync error handling and retry logic

Property Tests:
- Sync round-trip: Data synced from CLI should be retrievable from Supabase
- Sync idempotency: Syncing the same data twice should not create duplicates

### Test Implementation Examples

**Property Test Example (TypeScript with fast-check):**
```typescript
import fc from 'fast-check';

// Feature: content-and-backend-integration, Property 4: Recent sessions limit
test('Recent sessions should respect limit and ordering', async () => {
  await fc.assert(
    fc.asyncProperty(
      fc.array(fc.record({
        task_id: fc.string(),
        developer: fc.string(),
        start_time: fc.integer({ min: 0 }),
        end_time: fc.integer({ min: 0 }),
        lines_added: fc.nat(),
        lines_deleted: fc.nat(),
      }), { minLength: 0, maxLength: 20 }),
      fc.integer({ min: 1, max: 10 }),
      async (sessions, limit) => {
        // Insert test sessions
        await insertTestSessions(sessions);
        
        // Fetch with limit
        const result = await fetchRecentSessions(limit);
        
        // Verify limit
        expect(result.length).toBeLessThanOrEqual(limit);
        
        // Verify ordering (descending by start_time)
        for (let i = 0; i < result.length - 1; i++) {
          expect(result[i].start_time).toBeGreaterThanOrEqual(result[i + 1].start_time);
        }
      }
    ),
    { numRuns: 100 }
  );
});
```

**Unit Test Example (TypeScript with Jest):**
```typescript
// Test empty state handling
test('Dashboard should show empty state when no sessions exist', async () => {
  // Mock empty data
  mockSupabaseQuery.mockResolvedValue({ data: [], error: null });
  
  // Render dashboard
  render(<DashboardPage />);
  
  // Verify empty state is displayed
  expect(screen.getByText(/no sessions yet/i)).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /get started/i })).toBeInTheDocument();
});
```

### Integration Testing

**End-to-End Tests:**
- Test complete user flows (sign-in → view dashboard → view health metrics)
- Test CLI sync → dashboard display flow
- Test real-time updates from CLI to dashboard

**API Integration Tests:**
- Test Supabase RLS policies with different user contexts
- Test CLI sync endpoint with valid and invalid payloads
- Test authentication middleware

### Performance Testing

**Load Testing:**
- Test dashboard performance with large datasets (10,000+ sessions)
- Test dependency graph rendering with 1,000+ nodes
- Test real-time subscription performance with multiple concurrent users

**Optimization Targets:**
- Dashboard initial load: < 2 seconds
- Data fetch operations: < 500ms
- Real-time update latency: < 1 second

### Test Coverage Goals

- Unit test coverage: > 80% for business logic
- Property test coverage: All correctness properties implemented
- Integration test coverage: All critical user flows
- E2E test coverage: Happy path and error scenarios
