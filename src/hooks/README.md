# React Hooks Documentation

This document provides examples of how to use the newly implemented React hooks for state management and API integration.

## useQueue Hook

Manages download queue state and operations.

```tsx
import React from 'react';
import { useQueue, useQueuePolling } from '../hooks';

const QueueExample: React.FC = () => {
  const {
    jobs,
    isPaused,
    stats,
    isLoading,
    error,
    addJob,
    retryJob,
    cancelJob,
    pauseQueue,
    resumeQueue,
    refreshQueue
  } = useQueue();

  // Auto-refresh queue every 2 seconds
  useQueuePolling(refreshQueue, { enabled: !isPaused });

  const handleAddJob = async () => {
    try {
      await addJob('https://music.youtube.com/watch?v=example');
    } catch (error) {
      // Error is automatically handled by the hook
      console.error('Failed to add job:', error);
    }
  };

  return (
    <div>
      <h2>Queue ({stats.total} jobs)</h2>
      <button onClick={handleAddJob}>Add Job</button>
      <button onClick={isPaused ? resumeQueue : pauseQueue}>
        {isPaused ? 'Resume' : 'Pause'}
      </button>
      
      {isLoading && <p>Loading...</p>}
      {error && <p>Error: {error}</p>}
      
      <ul>
        {jobs.map(job => (
          <li key={job.id}>
            {job.url} - {job.status}
            <button onClick={() => retryJob(job.id)}>Retry</button>
            <button onClick={() => cancelJob(job.id)}>Cancel</button>
          </li>
        ))}
      </ul>
    </div>
  );
};
```

## useConfig Hook

Manages application configuration with validation and optimistic updates.

```tsx
import React from 'react';
import { useConfig } from '../hooks';

const ConfigExample: React.FC = () => {
  const {
    config,
    isLoading,
    isSaving,
    error,
    validationErrors,
    hasUnsavedChanges,
    updateConfig,
    saveConfig,
    resetConfig
  } = useConfig();

  if (isLoading) return <div>Loading configuration...</div>;
  if (!config) return <div>No configuration available</div>;

  const handleSave = async () => {
    try {
      await saveConfig();
    } catch (error) {
      // Error is automatically handled by the hook
      console.error('Failed to save config:', error);
    }
  };

  return (
    <div>
      <h2>Configuration</h2>
      
      <label>
        Output Path:
        <input
          value={config.output_path}
          onChange={(e) => updateConfig({ output_path: e.target.value })}
        />
      </label>
      
      <label>
        Audio Quality:
        <select
          value={config.itag}
          onChange={(e) => updateConfig({ itag: e.target.value })}
        >
          <option value="141">AAC 256kbps</option>
          <option value="140">AAC 128kbps</option>
        </select>
      </label>
      
      {validationErrors.map(error => (
        <p key={error.field} style={{ color: 'red' }}>
          {error.field}: {error.message}
        </p>
      ))}
      
      <div>
        <button onClick={handleSave} disabled={isSaving || !hasUnsavedChanges}>
          {isSaving ? 'Saving...' : 'Save'}
        </button>
        <button onClick={resetConfig} disabled={!hasUnsavedChanges}>
          Reset
        </button>
      </div>
    </div>
  );
};
```

## useCookies Hook

Manages YouTube Music cookies for authentication.

```tsx
import React from 'react';
import { useCookies } from '../hooks';
import { BrowserType } from '../types';

const CookieExample: React.FC = () => {
  const {
    validationResult,
    isLoading,
    isImporting,
    error,
    importCookies,
    validateCookies,
    clearCookies,
    getBrowserInstructions,
    checkCookieExpiration
  } = useCookies();

  const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      try {
        await importCookies(file.path);
      } catch (error) {
        console.error('Failed to import cookies:', error);
      }
    }
  };

  const chromeInstructions = getBrowserInstructions(BrowserType.Chrome);
  const isExpiringSoon = checkCookieExpiration();

  return (
    <div>
      <h2>Cookie Management</h2>
      
      {isExpiringSoon && (
        <div style={{ color: 'orange' }}>
          ⚠️ Cookies are expiring soon!
        </div>
      )}
      
      <div>
        <h3>Import Cookies</h3>
        <input type="file" accept=".txt" onChange={handleFileSelect} />
        {isImporting && <p>Importing cookies...</p>}
      </div>
      
      <div>
        <h3>Cookie Status</h3>
        {isLoading ? (
          <p>Validating cookies...</p>
        ) : validationResult ? (
          <div>
            <p>Valid: {validationResult.is_valid ? '✅' : '❌'}</p>
            {validationResult.expiration_date && (
              <p>Expires: {validationResult.expiration_date}</p>
            )}
            <p>Has PO Token: {validationResult.has_po_token ? '✅' : '❌'}</p>
          </div>
        ) : (
          <p>No cookies loaded</p>
        )}
      </div>
      
      <div>
        <button onClick={validateCookies}>Validate Cookies</button>
        <button onClick={clearCookies}>Clear Cookies</button>
      </div>
      
      <div>
        <h3>Chrome Instructions</h3>
        <ol>
          {chromeInstructions.steps.map((step, index) => (
            <li key={index}>{step}</li>
          ))}
        </ol>
      </div>
    </div>
  );
};
```

## usePolling Hook

Provides real-time updates for dynamic data.

```tsx
import React from 'react';
import { usePolling, useQueuePolling, useProgressPolling } from '../hooks';

const PollingExample: React.FC = () => {
  const [data, setData] = React.useState(null);
  const [isEnabled, setIsEnabled] = React.useState(true);

  // Generic polling
  const { start, stop, isPolling } = usePolling(
    async () => {
      // Fetch some data
      const response = await fetch('/api/data');
      const newData = await response.json();
      setData(newData);
    },
    {
      interval: 5000, // 5 seconds
      enabled: isEnabled,
      immediate: true
    }
  );

  // Specialized queue polling
  const refreshQueue = React.useCallback(async () => {
    // Queue refresh logic
  }, []);

  useQueuePolling(refreshQueue, { enabled: isEnabled });

  return (
    <div>
      <h2>Polling Example</h2>
      <p>Polling Status: {isPolling ? 'Active' : 'Inactive'}</p>
      
      <button onClick={() => setIsEnabled(!isEnabled)}>
        {isEnabled ? 'Disable' : 'Enable'} Polling
      </button>
      
      <button onClick={start}>Start Manual Polling</button>
      <button onClick={stop}>Stop Manual Polling</button>
      
      <pre>{JSON.stringify(data, null, 2)}</pre>
    </div>
  );
};
```

## Error Handling and Loading States

The hooks automatically integrate with the error handler and loading state manager:

```tsx
import React from 'react';
import { useErrorHandler, useLoadingState } from '../services';

const StatusExample: React.FC = () => {
  const { handleSuccess, handleWarning, handleApiError } = useErrorHandler();
  const { isLoading, setLoading } = useLoadingState();

  const performOperation = async () => {
    setLoading('my-operation', true);
    
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000));
      handleSuccess('Operation completed successfully!');
    } catch (error) {
      handleApiError(error, 'My Operation');
    } finally {
      setLoading('my-operation', false);
    }
  };

  return (
    <div>
      <button onClick={performOperation} disabled={isLoading('my-operation')}>
        {isLoading('my-operation') ? 'Processing...' : 'Start Operation'}
      </button>
    </div>
  );
};
```

## Integration with Components

The hooks are designed to work seamlessly with the existing components. Simply replace direct Tauri invoke calls with the appropriate hooks:

```tsx
// Before (direct Tauri calls)
import { invoke } from '@tauri-apps/api/core';

const oldAddJob = async (url: string) => {
  try {
    const result = await invoke('add_to_queue', { url });
    // Handle result...
  } catch (error) {
    // Handle error...
  }
};

// After (using hooks)
import { useQueue } from '../hooks';

const MyComponent = () => {
  const { addJob } = useQueue();
  
  const handleAddJob = async (url: string) => {
    try {
      await addJob(url); // Error handling and loading states are automatic
    } catch (error) {
      // Only handle component-specific logic if needed
    }
  };
  
  // Component JSX...
};
```

This approach provides:
- Automatic error handling with user-friendly notifications
- Loading state management with visual indicators
- Optimistic updates for better UX
- Type safety with TypeScript
- Consistent API across all operations
- Real-time updates with polling