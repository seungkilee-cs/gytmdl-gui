export interface AppConfig {
  // Paths
  output_path: string;
  temp_path: string;
  cookies_path?: string;
  
  // Download Settings
  itag: string;
  download_mode: DownloadMode;
  concurrent_limit: number;
  
  // Quality Settings
  cover_size: number;
  cover_format: CoverFormat;
  cover_quality: number;
  
  // Templates
  template_folder: string;
  template_file: string;
  template_date: string;
  
  // Advanced Options
  po_token?: string;
  exclude_tags?: string;
  truncate?: number;
  save_cover: boolean;
  overwrite: boolean;
  no_synced_lyrics: boolean;
}

export enum DownloadMode {
  Audio = "audio",
  Video = "video",
  AudioVideo = "audio_video",
}

export enum CoverFormat {
  Jpg = "jpg",
  Png = "png",
  Webp = "webp",
}

export interface ConfigValidationError {
  field: string;
  message: string;
}

export interface ConfigValidationResult {
  isValid: boolean;
  errors: ConfigValidationError[];
}