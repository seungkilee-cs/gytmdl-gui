use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use std::fs;
use std::io;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl AppState {
    /// Create a new AppState with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load AppState from a JSON file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path)?;
        let state: AppState = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(state)
    }

    /// Save AppState to a JSON file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), io::Error> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Add a new job to the queue
    pub fn add_job(&mut self, url: String) -> String {
        let job_id = Uuid::new_v4().to_string();
        let job = DownloadJob {
            id: job_id.clone(),
            url,
            status: JobStatus::Queued,
            progress: Progress::default(),
            metadata: None,
            error: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };
        self.jobs.push(job);
        job_id
    }

    /// Get a job by ID
    pub fn get_job(&self, job_id: &str) -> Option<&DownloadJob> {
        self.jobs.iter().find(|job| job.id == job_id)
    }

    /// Get a mutable reference to a job by ID
    pub fn get_job_mut(&mut self, job_id: &str) -> Option<&mut DownloadJob> {
        self.jobs.iter_mut().find(|job| job.id == job_id)
    }

    /// Update job status
    pub fn update_job_status(&mut self, job_id: &str, status: JobStatus) -> bool {
        if let Some(job) = self.get_job_mut(job_id) {
            job.status = status.clone();
            match status {
                JobStatus::Downloading => {
                    if job.started_at.is_none() {
                        job.started_at = Some(Utc::now());
                    }
                }
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
                    job.completed_at = Some(Utc::now());
                }
                _ => {}
            }
            true
        } else {
            false
        }
    }

    /// Update job progress
    pub fn update_job_progress(&mut self, job_id: &str, progress: Progress) -> bool {
        if let Some(job) = self.get_job_mut(job_id) {
            job.progress = progress;
            true
        } else {
            false
        }
    }

    /// Update job metadata
    pub fn update_job_metadata(&mut self, job_id: &str, metadata: JobMetadata) -> bool {
        if let Some(job) = self.get_job_mut(job_id) {
            job.metadata = Some(metadata);
            true
        } else {
            false
        }
    }

    /// Set job error
    pub fn set_job_error(&mut self, job_id: &str, error: String) -> bool {
        if let Some(job) = self.get_job_mut(job_id) {
            job.error = Some(error);
            job.status = JobStatus::Failed;
            job.completed_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    /// Remove a job from the queue
    pub fn remove_job(&mut self, job_id: &str) -> bool {
        let initial_len = self.jobs.len();
        self.jobs.retain(|job| job.id != job_id);
        self.jobs.len() != initial_len
    }

    /// Get jobs by status
    pub fn get_jobs_by_status(&self, status: &JobStatus) -> Vec<&DownloadJob> {
        self.jobs.iter().filter(|job| &job.status == status).collect()
    }

    /// Get count of jobs by status
    pub fn count_jobs_by_status(&self, status: &JobStatus) -> usize {
        self.jobs.iter().filter(|job| &job.status == status).count()
    }

    /// Clear completed and failed jobs
    pub fn clear_completed_jobs(&mut self) {
        self.jobs.retain(|job| !matches!(job.status, JobStatus::Completed | JobStatus::Failed));
    }

    /// Pause the queue
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume the queue
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Check if queue is paused
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            stage: DownloadStage::Initializing,
            percentage: None,
            current_step: "Initializing...".to_string(),
            total_steps: None,
            current_step_index: None,
        }
    }
}

impl DownloadJob {
    /// Create a new download job
    pub fn new(url: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            status: JobStatus::Queued,
            progress: Progress::default(),
            metadata: None,
            error: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Check if the job is in a terminal state (completed, failed, or cancelled)
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled)
    }

    /// Check if the job is active (downloading)
    pub fn is_active(&self) -> bool {
        matches!(self.status, JobStatus::Downloading)
    }

    /// Check if the job can be retried
    pub fn can_retry(&self) -> bool {
        matches!(self.status, JobStatus::Failed | JobStatus::Cancelled)
    }

    /// Reset job for retry
    pub fn reset_for_retry(&mut self) {
        self.status = JobStatus::Queued;
        self.progress = Progress::default();
        self.error = None;
        self.started_at = None;
        self.completed_at = None;
    }
}