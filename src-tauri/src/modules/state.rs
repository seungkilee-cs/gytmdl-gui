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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use tempfile::tempdir;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.jobs.len(), 0);
        assert_eq!(state.concurrent_limit, 3);
        assert!(!state.is_paused);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.output_path, PathBuf::from("./downloads"));
        assert_eq!(config.temp_path, PathBuf::from("./temp"));
        assert_eq!(config.itag, "141");
        assert_eq!(config.concurrent_limit, 3);
        assert_eq!(config.cover_size, 1400);
        assert_eq!(config.cover_quality, 95);
        assert!(config.save_cover);
        assert!(!config.overwrite);
        assert!(!config.no_synced_lyrics);
    }

    #[test]
    fn test_download_job_creation() {
        let url = "https://music.youtube.com/watch?v=test123".to_string();
        let job = DownloadJob::new(url.clone());
        
        assert_eq!(job.url, url);
        assert_eq!(job.status, JobStatus::Queued);
        assert!(job.metadata.is_none());
        assert!(job.error.is_none());
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert!(!job.id.is_empty());
    }

    #[test]
    fn test_download_job_states() {
        let mut job = DownloadJob::new("https://test.com".to_string());
        
        // Initial state
        assert!(!job.is_terminal());
        assert!(!job.is_active());
        assert!(!job.can_retry());
        
        // Downloading state
        job.status = JobStatus::Downloading;
        assert!(!job.is_terminal());
        assert!(job.is_active());
        assert!(!job.can_retry());
        
        // Failed state
        job.status = JobStatus::Failed;
        assert!(job.is_terminal());
        assert!(!job.is_active());
        assert!(job.can_retry());
        
        // Completed state
        job.status = JobStatus::Completed;
        assert!(job.is_terminal());
        assert!(!job.is_active());
        assert!(!job.can_retry());
        
        // Cancelled state
        job.status = JobStatus::Cancelled;
        assert!(job.is_terminal());
        assert!(!job.is_active());
        assert!(job.can_retry());
    }

    #[test]
    fn test_download_job_reset_for_retry() {
        let mut job = DownloadJob::new("https://test.com".to_string());
        job.status = JobStatus::Failed;
        job.error = Some("Network error".to_string());
        job.started_at = Some(Utc::now());
        job.completed_at = Some(Utc::now());
        
        job.reset_for_retry();
        
        assert_eq!(job.status, JobStatus::Queued);
        assert!(job.error.is_none());
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
    }

    #[test]
    fn test_app_state_add_job() {
        let mut state = AppState::new();
        let url = "https://music.youtube.com/watch?v=test123".to_string();
        
        let job_id = state.add_job(url.clone());
        
        assert_eq!(state.jobs.len(), 1);
        assert!(!job_id.is_empty());
        
        let job = state.get_job(&job_id).unwrap();
        assert_eq!(job.url, url);
        assert_eq!(job.status, JobStatus::Queued);
    }

    #[test]
    fn test_app_state_get_job() {
        let mut state = AppState::new();
        let job_id = state.add_job("https://test.com".to_string());
        
        // Test getting existing job
        assert!(state.get_job(&job_id).is_some());
        
        // Test getting non-existent job
        assert!(state.get_job("non-existent-id").is_none());
    }

    #[test]
    fn test_app_state_update_job_status() {
        let mut state = AppState::new();
        let job_id = state.add_job("https://test.com".to_string());
        
        // Update to downloading
        assert!(state.update_job_status(&job_id, JobStatus::Downloading));
        let job = state.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Downloading);
        assert!(job.started_at.is_some());
        
        // Update to completed
        assert!(state.update_job_status(&job_id, JobStatus::Completed));
        let job = state.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
        
        // Test updating non-existent job
        assert!(!state.update_job_status("non-existent", JobStatus::Failed));
    }

    #[test]
    fn test_app_state_update_job_progress() {
        let mut state = AppState::new();
        let job_id = state.add_job("https://test.com".to_string());
        
        let progress = Progress {
            stage: DownloadStage::DownloadingAudio,
            percentage: Some(50.0),
            current_step: "Downloading...".to_string(),
            total_steps: Some(5),
            current_step_index: Some(3),
        };
        
        assert!(state.update_job_progress(&job_id, progress.clone()));
        let job = state.get_job(&job_id).unwrap();
        assert_eq!(job.progress.percentage, Some(50.0));
        assert_eq!(job.progress.current_step, "Downloading...");
        
        // Test updating non-existent job
        assert!(!state.update_job_progress("non-existent", progress));
    }

    #[test]
    fn test_app_state_update_job_metadata() {
        let mut state = AppState::new();
        let job_id = state.add_job("https://test.com".to_string());
        
        let metadata = JobMetadata {
            title: Some("Test Song".to_string()),
            artist: Some("Test Artist".to_string()),
            album: Some("Test Album".to_string()),
            duration: Some(180),
            thumbnail: Some("https://thumbnail.url".to_string()),
        };
        
        assert!(state.update_job_metadata(&job_id, metadata.clone()));
        let job = state.get_job(&job_id).unwrap();
        assert_eq!(job.metadata.as_ref().unwrap().title, Some("Test Song".to_string()));
        assert_eq!(job.metadata.as_ref().unwrap().artist, Some("Test Artist".to_string()));
        
        // Test updating non-existent job
        assert!(!state.update_job_metadata("non-existent", metadata));
    }

    #[test]
    fn test_app_state_set_job_error() {
        let mut state = AppState::new();
        let job_id = state.add_job("https://test.com".to_string());
        
        let error_msg = "Network timeout".to_string();
        assert!(state.set_job_error(&job_id, error_msg.clone()));
        
        let job = state.get_job(&job_id).unwrap();
        assert_eq!(job.error, Some(error_msg));
        assert_eq!(job.status, JobStatus::Failed);
        assert!(job.completed_at.is_some());
        
        // Test setting error on non-existent job
        assert!(!state.set_job_error("non-existent", "Error".to_string()));
    }

    #[test]
    fn test_app_state_remove_job() {
        let mut state = AppState::new();
        let job_id = state.add_job("https://test.com".to_string());
        
        assert_eq!(state.jobs.len(), 1);
        assert!(state.remove_job(&job_id));
        assert_eq!(state.jobs.len(), 0);
        
        // Test removing non-existent job
        assert!(!state.remove_job("non-existent"));
    }

    #[test]
    fn test_app_state_get_jobs_by_status() {
        let mut state = AppState::new();
        let job_id1 = state.add_job("https://test1.com".to_string());
        let job_id2 = state.add_job("https://test2.com".to_string());
        let job_id3 = state.add_job("https://test3.com".to_string());
        
        state.update_job_status(&job_id1, JobStatus::Downloading);
        state.update_job_status(&job_id2, JobStatus::Completed);
        // job_id3 remains Queued
        
        let queued_jobs = state.get_jobs_by_status(&JobStatus::Queued);
        let downloading_jobs = state.get_jobs_by_status(&JobStatus::Downloading);
        let completed_jobs = state.get_jobs_by_status(&JobStatus::Completed);
        
        assert_eq!(queued_jobs.len(), 1);
        assert_eq!(downloading_jobs.len(), 1);
        assert_eq!(completed_jobs.len(), 1);
        assert_eq!(queued_jobs[0].id, job_id3);
        assert_eq!(downloading_jobs[0].id, job_id1);
        assert_eq!(completed_jobs[0].id, job_id2);
    }

    #[test]
    fn test_app_state_count_jobs_by_status() {
        let mut state = AppState::new();
        let job_id1 = state.add_job("https://test1.com".to_string());
        let job_id2 = state.add_job("https://test2.com".to_string());
        
        assert_eq!(state.count_jobs_by_status(&JobStatus::Queued), 2);
        assert_eq!(state.count_jobs_by_status(&JobStatus::Downloading), 0);
        
        state.update_job_status(&job_id1, JobStatus::Downloading);
        assert_eq!(state.count_jobs_by_status(&JobStatus::Queued), 1);
        assert_eq!(state.count_jobs_by_status(&JobStatus::Downloading), 1);
    }

    #[test]
    fn test_app_state_clear_completed_jobs() {
        let mut state = AppState::new();
        let job_id1 = state.add_job("https://test1.com".to_string());
        let job_id2 = state.add_job("https://test2.com".to_string());
        let job_id3 = state.add_job("https://test3.com".to_string());
        let job_id4 = state.add_job("https://test4.com".to_string());
        
        state.update_job_status(&job_id1, JobStatus::Completed);
        state.update_job_status(&job_id2, JobStatus::Failed);
        state.update_job_status(&job_id3, JobStatus::Downloading);
        // job_id4 remains Queued
        
        assert_eq!(state.jobs.len(), 4);
        state.clear_completed_jobs();
        assert_eq!(state.jobs.len(), 2);
        
        // Only downloading and queued jobs should remain
        let remaining_jobs: Vec<&JobStatus> = state.jobs.iter().map(|j| &j.status).collect();
        assert!(remaining_jobs.contains(&&JobStatus::Downloading));
        assert!(remaining_jobs.contains(&&JobStatus::Queued));
        assert!(!remaining_jobs.contains(&&JobStatus::Completed));
        assert!(!remaining_jobs.contains(&&JobStatus::Failed));
    }

    #[test]
    fn test_app_state_pause_resume() {
        let mut state = AppState::new();
        
        assert!(!state.is_paused());
        
        state.pause();
        assert!(state.is_paused());
        
        state.resume();
        assert!(!state.is_paused());
    }

    #[test]
    fn test_app_state_serialization() {
        let mut state = AppState::new();
        let job_id = state.add_job("https://test.com".to_string());
        state.update_job_status(&job_id, JobStatus::Downloading);
        
        // Test serialization
        let serialized = serde_json::to_string(&state).expect("Failed to serialize");
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized: AppState = serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.jobs.len(), state.jobs.len());
        assert_eq!(deserialized.is_paused, state.is_paused);
        assert_eq!(deserialized.concurrent_limit, state.concurrent_limit);
        assert_eq!(deserialized.jobs[0].url, state.jobs[0].url);
        assert_eq!(deserialized.jobs[0].status, state.jobs[0].status);
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig::default();
        
        // Test serialization
        let serialized = serde_json::to_string(&config).expect("Failed to serialize config");
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized: AppConfig = serde_json::from_str(&serialized).expect("Failed to deserialize config");
        assert_eq!(deserialized.output_path, config.output_path);
        assert_eq!(deserialized.itag, config.itag);
        assert_eq!(deserialized.concurrent_limit, config.concurrent_limit);
        assert_eq!(deserialized.cover_size, config.cover_size);
        assert_eq!(deserialized.save_cover, config.save_cover);
    }

    #[test]
    fn test_app_state_file_operations() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_state.json");
        
        let mut original_state = AppState::new();
        let job_id = original_state.add_job("https://test.com".to_string());
        original_state.update_job_status(&job_id, JobStatus::Downloading);
        original_state.pause();
        
        // Test saving to file
        original_state.save_to_file(&file_path).expect("Failed to save state");
        assert!(file_path.exists());
        
        // Test loading from file
        let loaded_state = AppState::load_from_file(&file_path).expect("Failed to load state");
        assert_eq!(loaded_state.jobs.len(), original_state.jobs.len());
        assert_eq!(loaded_state.is_paused, original_state.is_paused);
        assert_eq!(loaded_state.jobs[0].url, original_state.jobs[0].url);
        assert_eq!(loaded_state.jobs[0].status, original_state.jobs[0].status);
    }

    #[test]
    fn test_app_state_file_operations_invalid_path() {
        let invalid_path = PathBuf::from("/invalid/path/that/does/not/exist/state.json");
        let state = AppState::new();
        
        // Test loading from non-existent file
        assert!(AppState::load_from_file(&invalid_path).is_err());
    }

    #[test]
    fn test_progress_default() {
        let progress = Progress::default();
        assert!(matches!(progress.stage, DownloadStage::Initializing));
        assert_eq!(progress.current_step, "Initializing...");
        assert!(progress.percentage.is_none());
        assert!(progress.total_steps.is_none());
        assert!(progress.current_step_index.is_none());
    }

    #[test]
    fn test_job_metadata_serialization() {
        let metadata = JobMetadata {
            title: Some("Test Song".to_string()),
            artist: Some("Test Artist".to_string()),
            album: Some("Test Album".to_string()),
            duration: Some(180),
            thumbnail: Some("https://thumbnail.url".to_string()),
        };
        
        let serialized = serde_json::to_string(&metadata).expect("Failed to serialize metadata");
        let deserialized: JobMetadata = serde_json::from_str(&serialized).expect("Failed to deserialize metadata");
        
        assert_eq!(deserialized.title, metadata.title);
        assert_eq!(deserialized.artist, metadata.artist);
        assert_eq!(deserialized.album, metadata.album);
        assert_eq!(deserialized.duration, metadata.duration);
        assert_eq!(deserialized.thumbnail, metadata.thumbnail);
    }

    // Thread safety tests
    #[test]
    fn test_app_state_thread_safety() {
        let state = Arc::new(Mutex::new(AppState::new()));
        let mut handles = vec![];
        
        // Spawn multiple threads that add jobs concurrently
        for i in 0..10 {
            let state_clone = Arc::clone(&state);
            let handle = thread::spawn(move || {
                let mut state_guard = state_clone.lock().unwrap();
                let url = format!("https://test{}.com", i);
                state_guard.add_job(url);
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all jobs were added
        let final_state = state.lock().unwrap();
        assert_eq!(final_state.jobs.len(), 10);
    }

    #[test]
    fn test_concurrent_job_status_updates() {
        let state = Arc::new(Mutex::new(AppState::new()));
        
        // Add jobs first
        let job_ids: Vec<String> = {
            let mut state_guard = state.lock().unwrap();
            (0..5).map(|i| {
                let url = format!("https://test{}.com", i);
                state_guard.add_job(url)
            }).collect()
        };
        
        let mut handles = vec![];
        
        // Spawn threads to update job statuses concurrently
        for (i, job_id) in job_ids.iter().enumerate() {
            let state_clone = Arc::clone(&state);
            let job_id_clone = job_id.clone();
            let handle = thread::spawn(move || {
                let mut state_guard = state_clone.lock().unwrap();
                let status = if i % 2 == 0 { JobStatus::Downloading } else { JobStatus::Completed };
                state_guard.update_job_status(&job_id_clone, status);
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify updates were applied correctly
        let final_state = state.lock().unwrap();
        let downloading_count = final_state.count_jobs_by_status(&JobStatus::Downloading);
        let completed_count = final_state.count_jobs_by_status(&JobStatus::Completed);
        
        assert_eq!(downloading_count, 3); // jobs 0, 2, 4
        assert_eq!(completed_count, 2);   // jobs 1, 3
    }

    #[test]
    fn test_concurrent_config_access() {
        let config = Arc::new(Mutex::new(AppConfig::default()));
        let mut handles = vec![];
        
        // Spawn threads that read and modify config concurrently
        for i in 0..5 {
            let config_clone = Arc::clone(&config);
            let handle = thread::spawn(move || {
                let mut config_guard = config_clone.lock().unwrap();
                config_guard.concurrent_limit = i + 1;
                config_guard.cover_size = 1000 + (i * 100) as u32;
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final state is consistent (last thread wins)
        let final_config = config.lock().unwrap();
        assert!(final_config.concurrent_limit >= 1 && final_config.concurrent_limit <= 5);
        assert!(final_config.cover_size >= 1000 && final_config.cover_size <= 1400);
    }
}