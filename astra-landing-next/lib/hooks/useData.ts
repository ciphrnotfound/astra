// Data Fetching Hooks for Astra Dashboard
'use client';

import { useState, useEffect, useCallback } from 'react';
import { createClient } from '@/lib/supabase/client';
import type {
  CLIDashboardStats,
  HealthMetricsWithTrends,
  SecurityIssue,
  CLITimelineEvent,
  CLITask,
  Dependency,
  AstraSession,
  SecurityIssueFilters,
  CLITimelineEventFilters,
  CLITaskFilters,
  AstraSessionFilters,
  PaginatedResponse,
} from '@/lib/db/types';

interface UseDataResult<T> {
  data: T | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

interface UsePaginatedDataResult<T> {
  data: T[];
  loading: boolean;
  error: Error | null;
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  } | null;
  refetch: () => Promise<void>;
  setPage: (page: number) => void;
}

export function useDashboardStats(userId: number | null): UseDataResult<CLIDashboardStats> {
  const [data, setData] = useState<CLIDashboardStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = useCallback(async () => {
    if (!userId) {
      setData(null);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const response = await fetch(`/api/dashboard/stats?userId=${userId}`);
      if (!response.ok) throw new Error('Failed to fetch dashboard stats');
      const result = await response.json();
      setData(result);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

export function useHealthMetrics(
  projectId: number | null,
  timeRange: '7d' | '30d' | '90d' = '30d'
): UseDataResult<HealthMetricsWithTrends> {
  const [data, setData] = useState<HealthMetricsWithTrends | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = useCallback(async () => {
    if (!projectId) {
      setData(null);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const response = await fetch(`/api/health?projectId=${projectId}&timeRange=${timeRange}`);
      if (!response.ok) throw new Error('Failed to fetch health metrics');
      const result = await response.json();
      setData(result);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [projectId, timeRange]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { data, loading, error, refetch: fetchData };
}

export function useRealtime(
  table: string,
  projectId: number | null,
  onUpdate: () => void,
  enabled: boolean = true
) {
  useEffect(() => {
    if (!enabled || !projectId) return;

    const supabase = createClient();
    const channel = supabase
      .channel(`${table}-changes`)
      .on(
        'postgres_changes',
        {
          event: '*',
          schema: 'public',
          table: table,
          filter: `project_id=eq.${projectId}`,
        },
        () => {
          onUpdate();
        }
      )
      .subscribe();

    return () => {
      supabase.removeChannel(channel);
    };
  }, [table, projectId, onUpdate, enabled]);
}
