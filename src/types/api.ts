// Common API response types
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface ApiError {
  message: string;
  code?: string;
  details?: any;
}

// Tauri command result types
export type TauriResult<T> = Promise<T>;

// Event types for real-time updates
export interface JobUpdateEvent {
  job_id: string;
  status?: string;
  progress?: any;
  error?: string;
}

export interface QueueUpdateEvent {
  type: 'job_added' | 'job_updated' | 'job_removed' | 'queue_paused' | 'queue_resumed';
  data?: any;
}