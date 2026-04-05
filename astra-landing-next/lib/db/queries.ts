// Database Query Utilities for Astra Landing Application
// Reusable query functions for fetching data from Supabase

import { createServerClient } from '../supabase/server';
import type { TeamDataWithMembers } from './schema';
import type {
  AstraSession,
  HealthSnapshot,
  SecurityIssue,
  CLITimelineEvent,
  CLITask,
  Dependency,
  CLIDashboardStats,
  HealthMetricsWithTrends,
  HealthTrendPoint,
  AstraSessionFilters,
  HealthSnapshotFilters,
  SecurityIssueFilters,
  CLITimelineEventFilters,
  CLITaskFilters,
  PaginationParams,
  PaginatedResponse,
} from './types';

// ============================================================================
// Dashboard Stats Queries
// ============================================================================

/**
 * Fetch aggregated dashboard statistics for a user
 * Includes session counts, lines added/deleted, and recent sessions
 */
export async function getDashboardStats(userId: number): Promise<CLIDashboardStats> {
  const supabase = await createServerClient();

  // Get user's projects
  const { data: projects } = await supabase
    .from('projects')
    .select('id')
    .eq('user_id', userId);

  const projectIds = projects?.map(p => p.id) || [];

  // Get total sessions count (filter by project_id if available, otherwise by developer)
  let sessionsQuery = supabase
    .from('astra_sessions')
    .select('*', { count: 'exact', head: true });
  
  if (projectIds.length > 0) {
    sessionsQuery = sessionsQuery.in('project_id', projectIds);
  }
  
  const { count: totalSessions } = await sessionsQuery;

  // Get aggregated lines
  let aggregatesQuery = supabase
    .from('astra_sessions')
    .select('lines_added, lines_deleted');
  
  if (projectIds.length > 0) {
    aggregatesQuery = aggregatesQuery.in('project_id', projectIds);
  }
  
  const { data: aggregates } = await aggregatesQuery;

  const totalLinesAdded = aggregates?.reduce((sum, s) => sum + s.lines_added, 0) || 0;
  const totalLinesDeleted = aggregates?.reduce((sum, s) => sum + s.lines_deleted, 0) || 0;

  // Get recent sessions
  let recentSessionsQuery = supabase
    .from('astra_sessions')
    .select('*')
    .order('start_time', { ascending: false })
    .limit(5);
  
  if (projectIds.length > 0) {
    recentSessionsQuery = recentSessionsQuery.in('project_id', projectIds);
  }
  
  const { data: recentSessions } = await recentSessionsQuery;

  // Get latest health snapshot
  let latestHealthQuery = supabase
    .from('health_snapshots')
    .select('*')
    .order('timestamp', { ascending: false })
    .limit(1);
  
  if (projectIds.length > 0) {
    latestHealthQuery = latestHealthQuery.in('project_id', projectIds);
  }
  
  const { data: latestHealth } = await latestHealthQuery.single();

  return {
    totalSessions: totalSessions || 0,
    totalLinesAdded,
    totalLinesDeleted,
    activeProjects: projectIds.length,
    recentSessions: (recentSessions as AstraSession[]) || [],
    latestHealthSnapshot: latestHealth as HealthSnapshot | null,
  };
}

// ============================================================================
// Health Metrics Queries
// ============================================================================

/**
 * Fetch health metrics with trends for a project
 */
export async function getHealthMetrics(
  projectId: number,
  timeRange: '7d' | '30d' | '90d' = '30d'
): Promise<HealthMetricsWithTrends | null> {
  const supabase = await createServerClient();

  // Calculate timestamp for time range
  const now = Math.floor(Date.now() / 1000);
  const ranges = {
    '7d': 7 * 24 * 60 * 60,
    '30d': 30 * 24 * 60 * 60,
    '90d': 90 * 24 * 60 * 60,
  };
  const since = now - ranges[timeRange];

  // Get current health snapshot
  const { data: current } = await supabase
    .from('health_snapshots')
    .select('*')
    .eq('project_id', projectId)
    .order('timestamp', { ascending: false })
    .limit(1)
    .single();

  if (!current) return null;

  // Get trend data
  const { data: trendData } = await supabase
    .from('health_snapshots')
    .select('*')
    .eq('project_id', projectId)
    .gte('timestamp', since)
    .order('timestamp', { ascending: true });

  const trend: HealthTrendPoint[] = (trendData || []).map((snapshot: HealthSnapshot) => ({
    timestamp: snapshot.timestamp,
    scores: {
      code_quality: snapshot.code_quality,
      test_health: snapshot.test_health,
      cross_lang_drift: snapshot.cross_lang_drift,
      security_surface: snapshot.security_surface,
      git_health: snapshot.git_health,
      team_velocity: snapshot.team_velocity,
    },
  }));

  return {
    current: {
      code_quality: current.code_quality,
      test_health: current.test_health,
      cross_lang_drift: current.cross_lang_drift,
      security_surface: current.security_surface,
      git_health: current.git_health,
      team_velocity: current.team_velocity,
    },
    trend,
    details: {
      total_lines: current.total_lines,
      file_count: current.file_count,
      todo_count: current.todo_count,
      test_files: current.test_files,
      language_count: current.language_count,
      migration_count: current.migration_count,
      security_files: current.security_files,
    },
  };
}

// ============================================================================
// Security Issues Queries
// ============================================================================

/**
 * Fetch security issues with optional filtering
 */
export async function getSecurityIssues(
  projectId: number,
  filters?: SecurityIssueFilters
): Promise<SecurityIssue[]> {
  const supabase = await createServerClient();

  let query = supabase
    .from('security_issues')
    .select('*')
    .eq('project_id', projectId);

  if (filters?.severity && filters.severity.length > 0) {
    query = query.in('severity', filters.severity);
  }

  if (filters?.status && filters.status.length > 0) {
    query = query.in('status', filters.status);
  }

  query = query.order('detected_at', { ascending: false });

  const { data, error } = await query;

  if (error) throw error;
  return (data as SecurityIssue[]) || [];
}

/**
 * Update security issue status
 */
export async function updateSecurityIssueStatus(
  issueId: number,
  status: 'open' | 'resolved' | 'ignored'
): Promise<void> {
  const supabase = await createServerClient();

  const updates: any = { status };
  if (status === 'resolved') {
    updates.resolved_at = Math.floor(Date.now() / 1000);
  }

  const { error } = await supabase
    .from('security_issues')
    .update(updates)
    .eq('id', issueId);

  if (error) throw error;
}

// ============================================================================
// Timeline Events Queries
// ============================================================================

/**
 * Fetch timeline events with optional filtering and pagination
 */
export async function getTimelineEvents(
  projectId: number,
  filters?: CLITimelineEventFilters,
  pagination?: PaginationParams
): Promise<PaginatedResponse<CLITimelineEvent>> {
  const supabase = await createServerClient();

  let query = supabase
    .from('timeline_events')
    .select('*', { count: 'exact' })
    .eq('project_id', projectId);

  if (filters?.event_type && filters.event_type.length > 0) {
    query = query.in('event_type', filters.event_type);
  }

  if (filters?.since) {
    query = query.gte('timestamp', filters.since);
  }

  if (filters?.until) {
    query = query.lte('timestamp', filters.until);
  }

  query = query.order('timestamp', { ascending: false });

  if (pagination) {
    const { page, limit } = pagination;
    const from = (page - 1) * limit;
    const to = from + limit - 1;
    query = query.range(from, to);
  }

  const { data, error, count } = await query;

  if (error) throw error;

  const totalPages = pagination ? Math.ceil((count || 0) / pagination.limit) : 1;

  return {
    data: (data as CLITimelineEvent[]) || [],
    pagination: {
      page: pagination?.page || 1,
      limit: pagination?.limit || (data?.length || 0),
      total: count || 0,
      totalPages,
    },
  };
}

// ============================================================================
// Tasks Queries
// ============================================================================

/**
 * Fetch tasks with optional filtering
 */
export async function getCLITasks(
  projectId: number,
  filters?: CLITaskFilters
): Promise<CLITask[]> {
  const supabase = await createServerClient();

  let query = supabase
    .from('tasks')
    .select('*')
    .eq('project_id', projectId);

  if (filters?.status && filters.status.length > 0) {
    query = query.in('status', filters.status);
  }

  if (filters?.assignee) {
    query = query.eq('assignee', filters.assignee);
  }

  query = query.order('created_at', { ascending: false });

  const { data, error } = await query;

  if (error) throw error;
  return (data as CLITask[]) || [];
}

/**
 * Update task status
 */
export async function updateCLITaskStatus(
  taskId: string,
  status: 'Pending' | 'InProgress' | 'Done'
): Promise<void> {
  const supabase = await createServerClient();

  const { error } = await supabase
    .from('tasks')
    .update({ status, updated_at: new Date().toISOString() })
    .eq('task_id', taskId);

  if (error) throw error;
}

// ============================================================================
// Dependency Graph Queries
// ============================================================================

/**
 * Fetch dependency graph data for a project
 */
export async function getDependencyGraph(projectId: number): Promise<Dependency[]> {
  const supabase = await createServerClient();

  const { data, error } = await supabase
    .from('dependencies')
    .select('*')
    .eq('project_id', projectId);

  if (error) throw error;
  return (data as Dependency[]) || [];
}

// ============================================================================
// Astra Sessions Queries
// ============================================================================

/**
 * Fetch Astra sessions with optional filtering and pagination
 */
export async function getAstraSessions(
  projectId: number,
  filters?: AstraSessionFilters,
  pagination?: PaginationParams
): Promise<PaginatedResponse<AstraSession>> {
  const supabase = await createServerClient();

  let query = supabase
    .from('astra_sessions')
    .select('*', { count: 'exact' })
    .eq('project_id', projectId);

  if (filters?.developer) {
    query = query.eq('developer', filters.developer);
  }

  if (filters?.task_id) {
    query = query.eq('task_id', filters.task_id);
  }

  if (filters?.since) {
    query = query.gte('start_time', filters.since);
  }

  if (filters?.until) {
    query = query.lte('end_time', filters.until);
  }

  query = query.order('start_time', { ascending: false });

  if (pagination) {
    const { page, limit } = pagination;
    const from = (page - 1) * limit;
    const to = from + limit - 1;
    query = query.range(from, to);
  }

  const { data, error, count } = await query;

  if (error) throw error;

  const totalPages = pagination ? Math.ceil((count || 0) / pagination.limit) : 1;

  return {
    data: (data as AstraSession[]) || [],
    pagination: {
      page: pagination?.page || 1,
      limit: pagination?.limit || (data?.length || 0),
      total: count || 0,
      totalPages,
    },
  };
}

// ============================================================================
// Migration Queries
// ============================================================================

/**
 * Fetch migrations with pagination
 */
export async function getMigrations(
  userId: number,
  pagination?: PaginationParams
) {
  const supabase = await createServerClient();

  let query = supabase
    .from('migrations')
    .select('*', { count: 'exact' })
    .eq('user_id', userId)
    .order('created_at', { ascending: false });

  if (pagination) {
    const { page, limit } = pagination;
    const from = (page - 1) * limit;
    const to = from + limit - 1;
    query = query.range(from, to);
  }

  const { data, error, count } = await query;

  if (error) throw error;

  const totalPages = pagination ? Math.ceil((count || 0) / pagination.limit) : 1;

  return {
    data: data || [],
    pagination: {
      page: pagination?.page || 1,
      limit: pagination?.limit || (data?.length || 0),
      total: count || 0,
      totalPages,
    },
  };
}

// ============================================================================
// Authentication & Team Management Queries
// ============================================================================

/**
 * Get the currently authenticated user from session
 */
export async function getUser() {
  const supabase = await createServerClient();
  
  const { data: { user: authUser } } = await supabase.auth.getUser();
  
  if (!authUser) return null;

  const { data: user } = await supabase
    .from('users')
    .select('*')
    .eq('email', authUser.email)
    .is('deleted_at', null)
    .single();

  return user;
}

/**
 * Get the current user's team with all team members
 */
export async function getTeamForUser(): Promise<TeamDataWithMembers | null> {
  const user = await getUser();
  if (!user) return null;

  const supabase = await createServerClient();

  // Get user's team membership
  const { data: membership } = await supabase
    .from('team_members')
    .select(`
      team_id,
      teams (
        id,
        name,
        created_at,
        updated_at,
        stripe_customer_id,
        stripe_subscription_id,
        stripe_product_id,
        plan_name,
        subscription_status,
        team_members (
          id,
          role,
          joined_at,
          user_id,
          team_id,
          users (
            id,
            name,
            email
          )
        )
      )
    `)
    .eq('user_id', user.id)
    .single();

  if (!membership || !membership.teams) return null;

  const teamData: any = membership.teams;
  
  // Transform Supabase response to match TeamDataWithMembers type
  return {
    id: teamData.id,
    name: teamData.name,
    createdAt: new Date(teamData.created_at),
    updatedAt: new Date(teamData.updated_at),
    stripeCustomerId: teamData.stripe_customer_id,
    stripeSubscriptionId: teamData.stripe_subscription_id,
    stripeProductId: teamData.stripe_product_id,
    planName: teamData.plan_name,
    subscriptionStatus: teamData.subscription_status,
    teamMembers: (teamData.team_members || []).map((member: any) => ({
      id: member.id,
      userId: member.user_id,
      teamId: member.team_id,
      role: member.role,
      joinedAt: new Date(member.joined_at),
      user: {
        id: member.users.id,
        name: member.users.name,
        email: member.users.email,
      },
    })),
  };
}

/**
 * Get user with their team information by userId
 */
export async function getUserWithTeam(userId: number) {
  const supabase = await createServerClient();

  const { data: user } = await supabase
    .from('users')
    .select(`
      *,
      team_members (
        team_id
      )
    `)
    .eq('id', userId)
    .single();

  if (!user) return null;

  return {
    ...user,
    teamId: user.team_members?.[0]?.team_id || null,
  };
}

/**
 * Get team by Stripe customer ID
 */
export async function getTeamByStripeCustomerId(customerId: string) {
  const supabase = await createServerClient();

  const { data: team } = await supabase
    .from('teams')
    .select('*')
    .eq('stripe_customer_id', customerId)
    .single();

  return team;
}

/**
 * Update team subscription information
 */
export async function updateTeamSubscription(
  teamId: number,
  subscriptionData: {
    stripeSubscriptionId: string | null;
    stripeProductId: string | null;
    planName: string | null;
    subscriptionStatus: string;
  }
) {
  const supabase = await createServerClient();

  const { error } = await supabase
    .from('teams')
    .update({
      stripe_subscription_id: subscriptionData.stripeSubscriptionId,
      stripe_product_id: subscriptionData.stripeProductId,
      plan_name: subscriptionData.planName,
      subscription_status: subscriptionData.subscriptionStatus,
      updated_at: new Date().toISOString(),
    })
    .eq('id', teamId);

  if (error) throw error;
}
