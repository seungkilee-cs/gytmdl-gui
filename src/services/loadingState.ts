import React, { useState, useCallback } from 'react';

export interface LoadingState {
  [key: string]: boolean;
}

/**
 * Loading state manager for tracking multiple async operations
 */
export class LoadingStateManager {
  private static instance: LoadingStateManager;
  private loadingStates: LoadingState = {};
  private listeners: ((states: LoadingState) => void)[] = [];

  private constructor() {}

  static getInstance(): LoadingStateManager {
    if (!LoadingStateManager.instance) {
      LoadingStateManager.instance = new LoadingStateManager();
    }
    return LoadingStateManager.instance;
  }

  /**
   * Subscribe to loading state updates
   */
  subscribe(listener: (states: LoadingState) => void): () => void {
    this.listeners.push(listener);
    
    // Return unsubscribe function
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Notify all listeners of state changes
   */
  private notifyListeners(): void {
    this.listeners.forEach(listener => listener({ ...this.loadingStates }));
  }

  /**
   * Set loading state for a specific operation
   */
  setLoading(key: string, isLoading: boolean): void {
    if (isLoading) {
      this.loadingStates[key] = true;
    } else {
      delete this.loadingStates[key];
    }
    this.notifyListeners();
  }

  /**
   * Check if a specific operation is loading
   */
  isLoading(key: string): boolean {
    return this.loadingStates[key] || false;
  }

  /**
   * Check if any operation is loading
   */
  isAnyLoading(): boolean {
    return Object.keys(this.loadingStates).length > 0;
  }

  /**
   * Get all current loading states
   */
  getLoadingStates(): LoadingState {
    return { ...this.loadingStates };
  }

  /**
   * Clear all loading states
   */
  clearAll(): void {
    this.loadingStates = {};
    this.notifyListeners();
  }
}

// Export singleton instance
export const loadingStateManager = LoadingStateManager.getInstance();

/**
 * React hook for managing loading states
 */
export const useLoadingState = () => {
  const [loadingStates, setLoadingStates] = useState<LoadingState>({});

  // Subscribe to loading state changes
  React.useEffect(() => {
    const unsubscribe = loadingStateManager.subscribe(setLoadingStates);
    
    // Set initial state
    setLoadingStates(loadingStateManager.getLoadingStates());
    
    return unsubscribe;
  }, []);

  const setLoading = useCallback((key: string, isLoading: boolean) => {
    loadingStateManager.setLoading(key, isLoading);
  }, []);

  const isLoading = useCallback((key: string) => {
    return loadingStateManager.isLoading(key);
  }, []);

  const isAnyLoading = useCallback(() => {
    return loadingStateManager.isAnyLoading();
  }, []);

  return {
    loadingStates,
    setLoading,
    isLoading,
    isAnyLoading,
  };
};

/**
 * Higher-order function to wrap async operations with loading state
 */
export const withLoadingState = <T extends any[], R>(
  key: string,
  asyncFn: (...args: T) => Promise<R>
) => {
  return async (...args: T): Promise<R> => {
    loadingStateManager.setLoading(key, true);
    try {
      const result = await asyncFn(...args);
      return result;
    } finally {
      loadingStateManager.setLoading(key, false);
    }
  };
};