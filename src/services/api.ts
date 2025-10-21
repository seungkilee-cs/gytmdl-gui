import { invoke } from '@tauri-apps/api/core';
import { 
  QueueState, 
  AppConfig, 
  CookieValidationResult,
  CookieImportRequest,
  CookieImportResult,
  AddJobRequest,
  AddJobResponse,
  ConfigValidationResult,
  ApiError
} from '../types';

/**
 * Custom error class for API operations
 */
export class ApiOperationError extends Error {
  constructor(
    message: string,
    public code?: string,
    public details?: any
  ) {
    super(message);
    this.name = 'ApiOperationError';
  }
}

/**
 * Wrapper for Tauri invoke calls with error handling and type safety
 */
async function invokeWithErrorHandling<T>(
  command: string,
  args?: Record<string, any>
): Promise<T> {
  try {
    const result = await invoke<T>(command, args);
    return result;
  } catch (error) {
    // Handle different error types
    if (typeof error === 'string') {
      throw new ApiOperationError(error);
    } else if (error instanceof Error) {
      throw new ApiOperationError(error.message);
    } else if (typeof error === 'object' && error !== null) {
      const apiError = error as ApiError;
      throw new ApiOperationError(
        apiError.message || 'Unknown API error',
        apiError.code,
        apiError.details
      );
    } else {
      throw new ApiOperationError('Unknown error occurred');
    }
  }
}

/**
 * Queue Management API
 */
export const queueApi = {
  /**
   * Get current queue state
   */
  async getQueue(): Promise<QueueState> {
    return invokeWithErrorHandling<QueueState>('get_queue');
  },

  /**
   * Add a new job to the queue
   */
  async addJob(url: string): Promise<AddJobResponse> {
    const request: AddJobRequest = { url };
    return invokeWithErrorHandling<AddJobResponse>('add_to_queue', request);
  },

  /**
   * Retry a failed job
   */
  async retryJob(jobId: string): Promise<void> {
    return invokeWithErrorHandling<void>('retry_job', { job_id: jobId });
  },

  /**
   * Cancel a running or queued job
   */
  async cancelJob(jobId: string): Promise<void> {
    return invokeWithErrorHandling<void>('cancel_job', { job_id: jobId });
  },

  /**
   * Remove a job from the queue
   */
  async removeJob(jobId: string): Promise<void> {
    return invokeWithErrorHandling<void>('remove_job', { job_id: jobId });
  },

  /**
   * Pause the entire queue
   */
  async pauseQueue(): Promise<void> {
    return invokeWithErrorHandling<void>('pause_queue');
  },

  /**
   * Resume the queue
   */
  async resumeQueue(): Promise<void> {
    return invokeWithErrorHandling<void>('resume_queue');
  },

  /**
   * Clear all completed jobs
   */
  async clearCompleted(): Promise<void> {
    return invokeWithErrorHandling<void>('clear_completed_jobs');
  },

  /**
   * Clear all jobs (completed, failed, cancelled)
   */
  async clearAll(): Promise<void> {
    return invokeWithErrorHandling<void>('clear_all_jobs');
  },
};

/**
 * Configuration Management API
 */
export const configApi = {
  /**
   * Get current configuration
   */
  async getConfig(): Promise<AppConfig> {
    return invokeWithErrorHandling<AppConfig>('get_config');
  },

  /**
   * Update configuration
   */
  async updateConfig(config: AppConfig): Promise<void> {
    return invokeWithErrorHandling<void>('update_config', { config });
  },

  /**
   * Validate configuration
   */
  async validateConfig(config: AppConfig): Promise<ConfigValidationResult> {
    return invokeWithErrorHandling<ConfigValidationResult>('validate_config', { config });
  },

  /**
   * Reset configuration to defaults
   */
  async resetConfig(): Promise<AppConfig> {
    return invokeWithErrorHandling<AppConfig>('reset_config');
  },

  /**
   * Get default configuration
   */
  async getDefaultConfig(): Promise<AppConfig> {
    return invokeWithErrorHandling<AppConfig>('get_default_config');
  },
};

/**
 * Cookie Management API
 */
export const cookieApi = {
  /**
   * Import cookies from a file
   */
  async importCookies(filePath: string): Promise<CookieImportResult> {
    const request: CookieImportRequest = { file_path: filePath };
    return invokeWithErrorHandling<CookieImportResult>('import_cookies', request);
  },

  /**
   * Validate current cookies
   */
  async validateCookies(): Promise<CookieValidationResult> {
    return invokeWithErrorHandling<CookieValidationResult>('validate_cookies');
  },

  /**
   * Clear current cookies
   */
  async clearCookies(): Promise<void> {
    return invokeWithErrorHandling<void>('clear_cookies');
  },

  /**
   * Get cookie file path
   */
  async getCookiePath(): Promise<string> {
    return invokeWithErrorHandling<string>('get_cookie_path');
  },
};

/**
 * System/Utility API
 */
export const systemApi = {
  /**
   * Get application version
   */
  async getVersion(): Promise<string> {
    return invokeWithErrorHandling<string>('get_app_version');
  },

  /**
   * Get gytmdl version
   */
  async getGytmdlVersion(): Promise<string> {
    return invokeWithErrorHandling<string>('get_gytmdl_version');
  },

  /**
   * Check for updates
   */
  async checkForUpdates(): Promise<{ hasUpdate: boolean; version?: string }> {
    return invokeWithErrorHandling<{ hasUpdate: boolean; version?: string }>('check_for_updates');
  },

  /**
   * Open file dialog
   */
  async openFileDialog(filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
    return invokeWithErrorHandling<string | null>('open_file_dialog', { filters });
  },

  /**
   * Open directory dialog
   */
  async openDirectoryDialog(): Promise<string | null> {
    return invokeWithErrorHandling<string | null>('open_directory_dialog');
  },

  /**
   * Show notification
   */
  async showNotification(title: string, body: string): Promise<void> {
    return invokeWithErrorHandling<void>('show_notification', { title, body });
  },
};

/**
 * Combined API object for easy access
 */
export const api = {
  queue: queueApi,
  config: configApi,
  cookies: cookieApi,
  system: systemApi,
};

export default api;