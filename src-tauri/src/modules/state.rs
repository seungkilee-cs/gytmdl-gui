use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub jobs: Vec<DownloadJob>,
    pub config: AppConfig,
    pub is_paused: bool,
    pub concurrent_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadJob {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub progress: Progress,
    pub metadata: Option<JobMetadata>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<u32>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub stage: DownloadStage,
    pub percentage: Option<f32>,
    pub current_step: String,
    pub total_steps: Option<u32>,
    pub current_step_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStage {
    Initializing,
    FetchingMetadata,
    DownloadingAudio,
    Remuxing,
    ApplyingTags,
    Finalizing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Paths
    pub output_path: PathBuf,
    pub temp_path: PathBuf,
    pub cookies_path: Option<PathBuf>,
    
    // Download Settings
    pub itag: String,
    pub download_mode: DownloadMode,
    pub concurrent_limit: usize,
    
    // Quality Settings
    pub cover_size: u32,
    pub cover_format: CoverFormat,
    pub cover_quality: u8,
    
    // Templates
    pub template_folder: String,
    pub template_file: String,
    pub template_date: String,
    
    // Advanced Options
    pub po_token: Option<String>,
    pub exclude_tags: Option<String>,
    pub truncate: Option<u32>,
    pub save_cover: bool,
    pub overwrite: bool,
    pub no_synced_lyrics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadMode {
    Audio,
    Video,
    AudioVideo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoverFormat {
    Jpg,
    Png,
    Webp,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            config: AppConfig::default(),
            is_paused: false,
            concurrent_limit: 3,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("./downloads"),
            temp_path: PathBuf::from("./temp"),
            cookies_path: None,
            itag: "141".to_string(),
            download_mode: DownloadMode::Audio,
            concurrent_limit: 3,
            cover_size: 1400,
            cover_format: CoverFormat::Jpg,
            cover_quality: 95,
            template_folder: "{album_artist}/{album}".to_string(),
            template_file: "{track:02d} {title}".to_string(),
            template_date: "%Y-%m-%d".to_string(),
            po_token: None,
            exclude_tags: None,
            truncate: None,
            save_cover: true,
            overwrite: false,
            no_synced_lyrics: false,
        }
    }
}