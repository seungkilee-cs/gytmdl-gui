import { useEffect, useRef, useCallback } from 'react';

export interface UsePollingOptions {
  interval?: number; // Polling interval in milliseconds (default: 2000)
  enabled?: boolean; // Whether polling is enabled (default: true)
  immediate?: boolean; // Whether to call the function immediately (default: true)
}

export interface UsePollingReturn {
  start: () => void;
  stop: () => void;
  isPolling: boolean;
}

export const usePolling = (
  callback: () => void | Promise<void>,
  options: UsePollingOptions = {}
): UsePollingReturn => {
  const {
    interval = 2000,
    enabled = true,
    immediate = true,
  } = options;

  const intervalRef = useRef<number | null>(null);
  const callbackRef = useRef(callback);
  const isPollingRef = useRef(false);

  // Update callback ref when callback changes
  useEffect(() => {
    callbackRef.current = callback;
  }, [callback]);

  // Start polling
  const start = useCallback(() => {
    if (isPollingRef.current) return;

    isPollingRef.current = true;

    // Call immediately if requested
    if (immediate) {
      try {
        const result = callbackRef.current();
        // Handle async callbacks
        if (result instanceof Promise) {
          result.catch(console.error);
        }
      } catch (error) {
        console.error('Polling callback error:', error);
      }
    }

    // Set up interval
    intervalRef.current = setInterval(async () => {
      try {
        const result = callbackRef.current();
        // Handle async callbacks
        if (result instanceof Promise) {
          await result;
        }
      } catch (error) {
        console.error('Polling callback error:', error);
      }
    }, interval);
  }, [interval, immediate]);

  // Stop polling
  const stop = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    isPollingRef.current = false;
  }, []);

  // Auto-start/stop based on enabled flag
  useEffect(() => {
    if (enabled) {
      start();
    } else {
      stop();
    }

    return stop; // Cleanup on unmount
  }, [enabled, start, stop]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      stop();
    };
  }, [stop]);

  return {
    start,
    stop,
    isPolling: isPollingRef.current,
  };
};

// Specialized hook for queue updates
export const useQueuePolling = (
  refreshQueue: () => void | Promise<void>,
  options: Omit<UsePollingOptions, 'interval'> & { interval?: number } = {}
) => {
  return usePolling(refreshQueue, {
    interval: 2000, // Default 2 seconds for queue updates
    ...options,
  });
};

// Specialized hook for progress updates
export const useProgressPolling = (
  updateProgress: () => void | Promise<void>,
  options: Omit<UsePollingOptions, 'interval'> & { interval?: number } = {}
) => {
  return usePolling(updateProgress, {
    interval: 1000, // Default 1 second for progress updates
    ...options,
  });
};