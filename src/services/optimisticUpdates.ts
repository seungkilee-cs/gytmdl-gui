import React from 'react';
import { DownloadJob, JobStatus } from '../types';

/**
 * Optimistic update manager for providing immediate UI feedback
 */
export class OptimisticUpdateManager {
  private static instance: OptimisticUpdateManager;
  private pendingUpdates: Map<string, any> = new Map();
  private listeners: ((updates: Map<string, any>) => void)[] = [];

  private constructor() {}

  static getInstance(): OptimisticUpdateManager {
    if (!OptimisticUpdateManager.instance) {
      OptimisticUpdateManager.instance = new OptimisticUpdateManager();
    }
    return OptimisticUpdateManager.instance;
  }

  /**
   * Subscribe to optimistic update changes
   */
  subscribe(listener: (updates: Map<string, any>) => void): () => void {
    this.listeners.push(listener);
    
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Notify all listeners of update changes
   */
  private notifyListeners(): void {
    this.listeners.forEach(listener => listener(new Map(this.pendingUpdates)));
  }

  /**
   * Add an optimistic update
   */
  addUpdate(key: string, update: any): void {
    this.pendingUpdates.set(key, update);
    this.notifyListeners();
  }

  /**
   * Remove an optimistic update (when real update arrives)
   */
  removeUpdate(key: string): void {
    this.pendingUpdates.delete(key);
    this.notifyListeners();
  }

  /**
   * Get a specific optimistic update
   */
  getUpdate(key: string): any {
    return this.pendingUpdates.get(key);
  }

  /**
   * Check if an update exists
   */
  hasUpdate(key: string): boolean {
    return this.pendingUpdates.has(key);
  }

  /**
   * Clear all optimistic updates
   */
  clearAll(): void {
    this.pendingUpdates.clear();
    this.notifyListeners();
  }
}

export const optimisticUpdateManager = OptimisticUpdateManager.getInstance();

/**
 * Optimistic update helpers for common operations
 */
export const optimisticUpdates = {
  /**
   * Apply optimistic job status update
   */
  updateJobStatus(jobId: string, status: JobStatus): void {
    const updateKey = `job-status-${jobId}`;
    optimisticUpdateManager.addUpdate(updateKey, { status });
    
    // Auto-remove after timeout to prevent stale updates
    setTimeout(() => {
      optimisticUpdateManager.removeUpdate(updateKey);
    }, 5000);
  },

  /**
   * Apply optimistic job addition
   */
  addJob(url: string): string {
    const tempId = `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const updateKey = `job-add-${tempId}`;
    
    const optimisticJob: Partial<DownloadJob> = {
      id: tempId,
      url,
      status: JobStatus.Queued,
      progress: {
        stage: 'initializing' as any,
        percentage: 0,
        current_step: 'Adding to queue...',
      },
      created_at: new Date().toISOString(),
    };
    
    optimisticUpdateManager.addUpdate(updateKey, optimisticJob);
    
    return tempId;
  },

  /**
   * Remove optimistic job addition
   */
  removeJobAddition(tempId: string): void {
    const updateKey = `job-add-${tempId}`;
    optimisticUpdateManager.removeUpdate(updateKey);
  },

  /**
   * Apply optimistic queue pause/resume
   */
  updateQueuePause(isPaused: boolean): void {
    const updateKey = 'queue-pause';
    optimisticUpdateManager.addUpdate(updateKey, { isPaused });
    
    // Auto-remove after timeout
    setTimeout(() => {
      optimisticUpdateManager.removeUpdate(updateKey);
    }, 3000);
  },

  /**
   * Apply optimistic config update
   */
  updateConfig(configChanges: any): void {
    const updateKey = 'config-update';
    optimisticUpdateManager.addUpdate(updateKey, configChanges);
    
    // Auto-remove after timeout
    setTimeout(() => {
      optimisticUpdateManager.removeUpdate(updateKey);
    }, 5000);
  },

  /**
   * Remove specific update
   */
  removeUpdate(key: string): void {
    optimisticUpdateManager.removeUpdate(key);
  },

  /**
   * Check if update exists
   */
  hasUpdate(key: string): boolean {
    return optimisticUpdateManager.hasUpdate(key);
  },
};

/**
 * React hook for using optimistic updates
 */
export const useOptimisticUpdates = () => {
  const [updates, setUpdates] = React.useState<Map<string, any>>(new Map());

  React.useEffect(() => {
    const unsubscribe = optimisticUpdateManager.subscribe(setUpdates);
    
    // Set initial state
    setUpdates(new Map(optimisticUpdateManager['pendingUpdates']));
    
    return unsubscribe;
  }, []);

  return {
    updates,
    addUpdate: optimisticUpdateManager.addUpdate.bind(optimisticUpdateManager),
    getUpdate: optimisticUpdateManager.getUpdate.bind(optimisticUpdateManager),
    ...optimisticUpdates,
  };
};