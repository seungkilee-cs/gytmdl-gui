import { DownloadJob } from './job';

export interface QueueState {
  jobs: DownloadJob[];
  is_paused: boolean;
  concurrent_limit: number;
}

export interface QueueStats {
  total: number;
  queued: number;
  downloading: number;
  completed: number;
  failed: number;
  cancelled: number;
}

export interface AddJobRequest {
  url: string;
}

export interface AddJobResponse {
  job_id: string;
  success: boolean;
  error?: string;
}