import { useState, useEffect, useCallback } from 'react';
import { DownloadJob, QueueStats } from '../types';
import { api } from '../services/api';
import { useErrorHandler } from '../services/errorHandler';
import { optimisticUpdates } from '../services/optimisticUpdates';
import { loadingStateManager } from '../services/loadingState';

export interface UseQueueReturn {
  // State
  jobs: DownloadJob[];
  isPaused: boolean;
  concurrentLimit: number;
  stats: QueueStats;
  isLoading: boolean;
  error: string | null;

  // Actions
  addJob: (url: string) => Promise<void>;
  retryJob: (jobId: string) => Promise<void>;
  cancelJob: (jobId: string) => Promise<void>;
  removeJob: (jobId: string) => Promise<void>;
  pauseQueue: () => Promise<void>;
  resumeQueue: () => Promise<void>;
  clearCompleted: () => Promise<void>;
  refreshQueue: () => Promise<void>;
}

export const useQueue = (): UseQueueReturn => {
  const [jobs, setJobs] = useState<DownloadJob[]>([]);
  const [isPaused, setIsPaused] = useState(false);
  const [concurrentLimit, setConcurrentLimit] = useState(3);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const { handleApiError, handleSuccess } = useErrorHandler();

  // Calculate queue statistics
  const stats: QueueStats = {
    total: jobs.length,
    queued: jobs.filter(job => job.status === 'queued').length,
    downloading: jobs.filter(job => job.status === 'downloading').length,
    completed: jobs.filter(job => job.status === 'completed').length,
    failed: jobs.filter(job => job.status === 'failed').length,
    cancelled: jobs.filter(job => job.status === 'cancelled').length,
  };

  // Fetch queue state from backend
  const refreshQueue = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      loadingStateManager.setLoading('queue-refresh', true);
      
      const queueState = await api.queue.getQueue();
      setJobs(queueState.jobs);
      setIsPaused(queueState.is_paused);
      setConcurrentLimit(queueState.concurrent_limit);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch queue';
      setError(errorMessage);
      handleApiError(err, 'Fetch Queue');
    } finally {
      setIsLoading(false);
      loadingStateManager.setLoading('queue-refresh', false);
    }
  }, [handleApiError]);

  // Add a new job to the queue
  const addJob = useCallback(async (url: string) => {
    // Apply optimistic update
    const tempId = optimisticUpdates.addJob(url);
    
    try {
      setError(null);
      loadingStateManager.setLoading('queue-add', true);
      
      const response = await api.queue.addJob(url);
      
      if (!response.success) {
        throw new Error(response.error || 'Failed to add job');
      }
      
      handleSuccess('Job added to queue successfully');
      
      // Remove optimistic update and refresh
      optimisticUpdates.removeJobAddition(tempId);
      await refreshQueue();
    } catch (err) {
      // Remove optimistic update on error
      optimisticUpdates.removeJobAddition(tempId);
      
      const errorMessage = err instanceof Error ? err.message : 'Failed to add job';
      setError(errorMessage);
      handleApiError(err, 'Add Job');
      throw err;
    } finally {
      loadingStateManager.setLoading('queue-add', false);
    }
  }, [refreshQueue, handleApiError, handleSuccess]);

  // Retry a failed job
  const retryJob = useCallback(async (jobId: string) => {
    // Apply optimistic update
    optimisticUpdates.updateJobStatus(jobId, 'queued' as any);
    
    try {
      setError(null);
      loadingStateManager.setLoading('queue-action', true);
      
      await api.queue.retryJob(jobId);
      handleSuccess('Job retry initiated');
      await refreshQueue();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to retry job';
      setError(errorMessage);
      handleApiError(err, 'Retry Job');
      throw err;
    } finally {
      loadingStateManager.setLoading('queue-action', false);
    }
  }, [refreshQueue, handleApiError, handleSuccess]);

  // Cancel a running or queued job
  const cancelJob = useCallback(async (jobId: string) => {
    // Apply optimistic update
    optimisticUpdates.updateJobStatus(jobId, 'cancelled' as any);
    
    try {
      setError(null);
      
      await api.queue.cancelJob(jobId);
      handleSuccess('Job cancelled');
      await refreshQueue();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to cancel job';
      setError(errorMessage);
      handleApiError(err, 'Cancel Job');
      throw err;
    }
  }, [refreshQueue, handleApiError, handleSuccess]);

  // Remove a job from the queue
  const removeJob = useCallback(async (jobId: string) => {
    try {
      setError(null);
      
      await api.queue.removeJob(jobId);
      handleSuccess('Job removed from queue');
      await refreshQueue();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to remove job';
      setError(errorMessage);
      handleApiError(err, 'Remove Job');
      throw err;
    }
  }, [refreshQueue, handleApiError, handleSuccess]);

  // Pause the entire queue
  const pauseQueue = useCallback(async () => {
    // Apply optimistic update
    optimisticUpdates.updateQueuePause(true);
    setIsPaused(true);
    
    try {
      setError(null);
      
      await api.queue.pauseQueue();
      handleSuccess('Queue paused');
    } catch (err) {
      // Revert optimistic update on error
      setIsPaused(false);
      
      const errorMessage = err instanceof Error ? err.message : 'Failed to pause queue';
      setError(errorMessage);
      handleApiError(err, 'Pause Queue');
      throw err;
    }
  }, [handleApiError, handleSuccess]);

  // Resume the queue
  const resumeQueue = useCallback(async () => {
    // Apply optimistic update
    optimisticUpdates.updateQueuePause(false);
    setIsPaused(false);
    
    try {
      setError(null);
      
      await api.queue.resumeQueue();
      handleSuccess('Queue resumed');
    } catch (err) {
      // Revert optimistic update on error
      setIsPaused(true);
      
      const errorMessage = err instanceof Error ? err.message : 'Failed to resume queue';
      setError(errorMessage);
      handleApiError(err, 'Resume Queue');
      throw err;
    }
  }, [handleApiError, handleSuccess]);

  // Clear all completed jobs
  const clearCompleted = useCallback(async () => {
    try {
      setError(null);
      
      await api.queue.clearCompleted();
      handleSuccess('Completed jobs cleared');
      await refreshQueue();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to clear completed jobs';
      setError(errorMessage);
      handleApiError(err, 'Clear Completed Jobs');
      throw err;
    }
  }, [refreshQueue, handleApiError, handleSuccess]);

  // Load initial queue state on mount
  useEffect(() => {
    refreshQueue();
  }, [refreshQueue]);

  return {
    // State
    jobs,
    isPaused,
    concurrentLimit,
    stats,
    isLoading,
    error,

    // Actions
    addJob,
    retryJob,
    cancelJob,
    removeJob,
    pauseQueue,
    resumeQueue,
    clearCompleted,
    refreshQueue,
  };
};