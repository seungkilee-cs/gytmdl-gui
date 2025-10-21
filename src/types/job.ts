export interface DownloadJob {
  id: string;
  url: string;
  status: JobStatus;
  progress: Progress;
  metadata?: JobMetadata;
  error?: string;
  created_at: string;
  started_at?: string;
  completed_at?: string;
}

export enum JobStatus {
  Queued = "queued",
  Downloading = "downloading",
  Completed = "completed",
  Failed = "failed",
  Cancelled = "cancelled",
}

export interface JobMetadata {
  title?: string;
  artist?: string;
  album?: string;
  duration?: number;
  thumbnail?: string;
}

export interface Progress {
  stage: DownloadStage;
  percentage?: number;
  current_step: string;
  total_steps?: number;
  current_step_index?: number;
}

export enum DownloadStage {
  Initializing = "initializing",
  FetchingMetadata = "fetching_metadata",
  DownloadingAudio = "downloading_audio",
  Remuxing = "remuxing",
  ApplyingTags = "applying_tags",
  Finalizing = "finalizing",
  Completed = "completed",
  Failed = "failed",
}