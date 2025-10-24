use crate::modules::state::{AppState, DownloadJob, JobStatus, Progress};
use crate::modules::gytmdl_wrapper::{GytmdlWrapper, GytmdlError};
use crate::modules::progress_parser::ProgressParser;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, RwLock};
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

/// Represents a job submission request
#[derive(Debug, Clone)]
pub struct JobSubmission {
    pub job_id: String,
    pub retry_count: u32,
}

/// Represents the result of a job execution
#[derive(Debug)]
pub enum JobResult {
    Success(String),
    Failed(String, String), // job_id, error_message
    Cancelled(String),
}

/// Manages the download queue with concurrent processing
pub struct QueueManager {
    state: Arc<RwLock<AppState>>,
    gytmdl_wrapper: Arc<GytmdlWrapper>,
    concurrent_limit: usize,
    job_sender: mpsc::UnboundedSender<JobSubmission>,
    job_receiver: Arc<Mutex<mpsc::UnboundedReceiver<JobSubmission>>>,
    worker_pool: Arc<Mutex<JoinSet<JobResult>>>,
    running_jobs: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
    is_paused: Arc<RwLock<bool>>,
    is_shutdown: Arc<RwLock<bool>>,
}

impl QueueManager {
    /// Create a new QueueManager with the specified concurrent limit
    pub fn new(state: Arc<RwLock<AppState>>, concurrent_limit: usize) -> Result<Self, GytmdlError> {
        let gytmdl_wrapper = Arc::new(GytmdlWrapper::new()?);
        let (job_sender, job_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            state,
            gytmdl_wrapper,
            concurrent_limit,
            job_sender,
            job_receiver: Arc::new(Mutex::new(job_receiver)),
            worker_pool: Arc::new(Mutex::new(JoinSet::new())),
            running_jobs: Arc::new(Mutex::new(HashMap::new())),
            is_paused: Arc::new(RwLock::new(false)),
            is_shutdown: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the queue manager processing loop
    pub async fn start(&self) -> Result<(), GytmdlError> {
        let state = Arc::clone(&self.state);
        let job_receiver = Arc::clone(&self.job_receiver);
        let _worker_pool = Arc::clone(&self.worker_pool);
        let running_jobs = Arc::clone(&self.running_jobs);
        let is_paused = Arc::clone(&self.is_paused);
        let is_shutdown = Arc::clone(&self.is_shutdown);
        let gytmdl_wrapper = Arc::clone(&self.gytmdl_wrapper);
        let concurrent_limit = self.concurrent_limit;

        tokio::spawn(async move {
            loop {
                // Check if we should shutdown
                if *is_shutdown.read().await {
                    break;
                }

                // Check if we're paused
                if *is_paused.read().await {
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // Check if we have capacity for more jobs
                let running_count = running_jobs.lock().await.len();
                if running_count >= concurrent_limit {
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // Try to get a job from the queue
                let job_submission = {
                    let mut receiver = job_receiver.lock().await;
                    receiver.recv().await
                };

                if let Some(submission) = job_submission {
                    // Get the job from state
                    let job = {
                        let state_guard = state.read().await;
                        state_guard.get_job(&submission.job_id).cloned()
                    };

                    if let Some(job) = job {
                        // Check if job is still in a valid state to process
                        if matches!(job.status, JobStatus::Queued) {
                            // Update job status to downloading
                            {
                                let mut state_guard = state.write().await;
                                state_guard.update_job_status(&job.id, JobStatus::Downloading);
                            }

                            // Spawn worker task
                            let job_handle = Self::spawn_worker_task(
                                Arc::clone(&state),
                                Arc::clone(&gytmdl_wrapper),
                                job,
                                submission.retry_count,
                            ).await;

                            // Store the job handle
                            running_jobs.lock().await.insert(submission.job_id.clone(), job_handle);
                        }
                    }
                } else {
                    // Channel closed, break the loop
                    break;
                }

                // Clean up completed jobs
                Self::cleanup_completed_jobs(Arc::clone(&running_jobs)).await;

                // Small delay to prevent busy waiting
                sleep(Duration::from_millis(10)).await;
            }

            // Cleanup all running jobs on shutdown
            Self::cleanup_all_jobs(Arc::clone(&running_jobs)).await;
        });

        Ok(())
    }

    /// Spawn a worker task for processing a download job
    async fn spawn_worker_task(
        state: Arc<RwLock<AppState>>,
        gytmdl_wrapper: Arc<GytmdlWrapper>,
        job: DownloadJob,
        retry_count: u32,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let job_id = job.id.clone();
            let result = Self::process_job(
                Arc::clone(&state),
                Arc::clone(&gytmdl_wrapper),
                job,
                retry_count,
            ).await;

            // Update job status based on result
            let mut state_guard = state.write().await;
            match result {
                JobResult::Success(_) => {
                    state_guard.update_job_status(&job_id, JobStatus::Completed);
                    state_guard.update_job_progress(&job_id, ProgressParser::create_completed_progress());
                }
                JobResult::Failed(_, error) => {
                    state_guard.set_job_error(&job_id, error);
                }
                JobResult::Cancelled(_) => {
                    state_guard.update_job_status(&job_id, JobStatus::Cancelled);
                }
            }
        })
    }

    /// Process a single download job
    async fn process_job(
        state: Arc<RwLock<AppState>>,
        gytmdl_wrapper: Arc<GytmdlWrapper>,
        job: DownloadJob,
        _retry_count: u32,
    ) -> JobResult {
        let job_id = job.id.clone();

        // Get current config
        let config = {
            let state_guard = state.read().await;
            state_guard.config.clone()
        };

        // Update progress to initializing
        {
            let mut state_guard = state.write().await;
            state_guard.update_job_progress(&job_id, ProgressParser::create_initializing_progress());
        }

        // Debug: Log the binary path and command being used
        println!("DEBUG: Attempting to spawn gytmdl process for job {}", job_id);
        println!("DEBUG: Binary path: {:?}", gytmdl_wrapper.get_binary_path());
        
        // Test binary first
        match gytmdl_wrapper.test_binary().await {
            Ok(version) => {
                println!("DEBUG: Binary test successful, version: {}", version);
            }
            Err(e) => {
                let error_msg = format!("Binary test failed: {}. Binary path: {:?}", e, gytmdl_wrapper.get_binary_path());
                println!("DEBUG: {}", error_msg);
                return JobResult::Failed(job_id, error_msg);
            }
        }

        // Spawn the gytmdl process
        let mut process = match gytmdl_wrapper.spawn_download_process(&config, &job).await {
            Ok(process) => {
                println!("DEBUG: Process spawned successfully with PID: {:?}", process.process_id());
                process
            },
            Err(e) => {
                let error_msg = match e {
                    crate::modules::gytmdl_wrapper::GytmdlError::BinaryNotFound(_) => {
                        format!("gytmdl binary not found. Please build sidecar binaries or install gytmdl. Error: {}", e)
                    }
                    _ => format!("Failed to spawn process: {}", e)
                };
                println!("DEBUG: Process spawn failed: {}", error_msg);
                return JobResult::Failed(job_id, error_msg);
            }
        };

        // Process output and update progress
        let mut stdout_done = false;
        let mut stderr_done = false;
        
        loop {
            // Check if process has finished first
            match process.try_wait() {
                Ok(Some(exit_status)) => {
                    println!("DEBUG: Process exited with status: {:?}", exit_status);
                    if exit_status.success() {
                        println!("DEBUG: Process completed successfully");
                        return JobResult::Success(job_id);
                    } else {
                        let error_msg = match exit_status.code() {
                            Some(2) => {
                                let msg = format!("gytmdl process failed with exit code 2. Binary path: {:?}. This usually means the binary is not working correctly or missing dependencies.", gytmdl_wrapper.get_binary_path());
                                println!("DEBUG: {}", msg);
                                msg
                            },
                            Some(code) => {
                                let msg = format!("Process exited with code: {}. Binary path: {:?}", code, gytmdl_wrapper.get_binary_path());
                                println!("DEBUG: {}", msg);
                                msg
                            },
                            None => {
                                let msg = format!("Process was terminated by signal. Binary path: {:?}", gytmdl_wrapper.get_binary_path());
                                println!("DEBUG: {}", msg);
                                msg
                            },
                        };
                        return JobResult::Failed(job_id, error_msg);
                    }
                }
                Ok(None) => {
                    // Process is still running, continue reading output
                }
                Err(e) => {
                    return JobResult::Failed(job_id, format!("Error checking process status: {}", e));
                }
            }

            // Read stdout if not done
            if !stdout_done {
                match process.read_stdout_line().await {
                    Ok(Some(line)) => {
                        println!("DEBUG: gytmdl stdout: {}", line);
                        let sanitized_line = ProgressParser::sanitize_output(&line);
                        
                        // Check for completion
                        if ProgressParser::is_completion_line(&sanitized_line) {
                            println!("DEBUG: Completion line detected: {}", sanitized_line);
                        }
                        
                        // Parse progress and update state
                        if let Some(progress) = ProgressParser::parse_output(&sanitized_line) {
                            println!("DEBUG: Progress parsed: {:?}", progress);
                            let mut state_guard = state.write().await;
                            state_guard.update_job_progress(&job_id, progress);
                        }
                    }
                    Ok(None) => {
                        println!("DEBUG: EOF on stdout");
                        stdout_done = true;
                    }
                    Err(e) => {
                        println!("DEBUG: Error reading stdout: {}", e);
                        return JobResult::Failed(job_id, format!("Error reading stdout: {}", e));
                    }
                }
            }

            // Read stderr if not done
            if !stderr_done {
                match process.read_stderr_line().await {
                    Ok(Some(line)) => {
                        println!("DEBUG: gytmdl stderr: {}", line);
                        let sanitized_line = ProgressParser::sanitize_output(&line);
                        
                        // Check for errors
                        if ProgressParser::is_error_line(&sanitized_line) {
                            println!("DEBUG: Error detected in stderr: {}", sanitized_line);
                            return JobResult::Failed(job_id, sanitized_line);
                        }
                        
                        // Parse progress from stderr as well
                        if let Some(progress) = ProgressParser::parse_output(&sanitized_line) {
                            let mut state_guard = state.write().await;
                            state_guard.update_job_progress(&job_id, progress);
                        }
                    }
                    Ok(None) => {
                        // EOF on stderr
                        stderr_done = true;
                    }
                    Err(e) => {
                        return JobResult::Failed(job_id, format!("Error reading stderr: {}", e));
                    }
                }
            }

            // If both streams are done, wait for process to finish
            if stdout_done && stderr_done {
                println!("DEBUG: Both stdout and stderr streams finished");
                break;
            }

            // Small delay to prevent busy waiting
            sleep(Duration::from_millis(10)).await;
        }

        // If we reach here, check the final process status
        match process.wait().await {
            Ok(exit_status) => {
                if exit_status.success() {
                    JobResult::Success(job_id)
                } else {
                    let error_msg = format!("Process exited with code: {:?}", exit_status.code());
                    JobResult::Failed(job_id, error_msg)
                }
            }
            Err(e) => {
                JobResult::Failed(job_id, format!("Error waiting for process: {}", e))
            }
        }
    }

    /// Submit a job to the queue for processing
    pub async fn submit_job(&self, job_id: String) -> Result<(), String> {
        let submission = JobSubmission {
            job_id,
            retry_count: 0,
        };

        self.job_sender.send(submission)
            .map_err(|e| format!("Failed to submit job: {}", e))?;

        Ok(())
    }

    /// Submit a job for retry with exponential backoff
    pub async fn retry_job(&self, job_id: String) -> Result<(), String> {
        let retry_count = {
            let mut state_guard = self.state.write().await;
            if let Some(job) = state_guard.get_job_mut(&job_id) {
                if !job.can_retry() {
                    return Err("Job cannot be retried".to_string());
                }
                
                // Get current retry count from job metadata or default to 0
                let current_retry_count = job.error.as_ref()
                    .and_then(|error| {
                        // Try to extract retry count from error message
                        if error.contains("retry_count:") {
                            error.split("retry_count:").nth(1)
                                .and_then(|s| s.split_whitespace().next())
                                .and_then(|s| s.parse::<u32>().ok())
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                
                let new_retry_count = current_retry_count + 1;
                
                // Check maximum retry limit
                if new_retry_count > 3 {
                    return Err("Maximum retry attempts exceeded".to_string());
                }
                
                job.reset_for_retry();
                new_retry_count
            } else {
                return Err("Job not found".to_string());
            }
        };

        // Apply exponential backoff delay
        let delay_ms = Self::calculate_backoff_delay(retry_count);
        if delay_ms > 0 {
            sleep(Duration::from_millis(delay_ms)).await;
        }

        let submission = JobSubmission {
            job_id,
            retry_count,
        };

        self.job_sender.send(submission)
            .map_err(|e| format!("Failed to submit retry job: {}", e))?;

        Ok(())
    }

    /// Calculate exponential backoff delay in milliseconds
    fn calculate_backoff_delay(retry_count: u32) -> u64 {
        // Base delay of 1 second, exponentially increasing
        let base_delay = 1000u64; // 1 second
        let max_delay = 30000u64; // 30 seconds max
        
        let delay = base_delay * (2u64.pow(retry_count.saturating_sub(1)));
        delay.min(max_delay)
    }

    /// Cancel a specific job
    pub async fn cancel_job(&self, job_id: &str) -> Result<(), String> {
        // Update job status to cancelled
        {
            let mut state_guard = self.state.write().await;
            state_guard.update_job_status(job_id, JobStatus::Cancelled);
        }

        // Kill the running process if it exists
        let mut running_jobs = self.running_jobs.lock().await;
        if let Some(handle) = running_jobs.remove(job_id) {
            handle.abort();
        }

        Ok(())
    }

    /// Pause the queue processing
    pub async fn pause(&self) {
        let mut is_paused = self.is_paused.write().await;
        *is_paused = true;

        // Update state
        let mut state_guard = self.state.write().await;
        state_guard.pause();
    }

    /// Resume the queue processing
    pub async fn resume(&self) {
        let mut is_paused = self.is_paused.write().await;
        *is_paused = false;

        // Update state
        let mut state_guard = self.state.write().await;
        state_guard.resume();
    }

    /// Check if the queue is paused
    pub async fn is_paused(&self) -> bool {
        *self.is_paused.read().await
    }

    /// Get the number of currently running jobs
    pub async fn running_count(&self) -> usize {
        self.running_jobs.lock().await.len()
    }

    /// Get the number of queued jobs
    pub async fn queued_count(&self) -> usize {
        let state_guard = self.state.read().await;
        state_guard.count_jobs_by_status(&JobStatus::Queued)
    }

    /// Shutdown the queue manager
    pub async fn shutdown(&self) {
        let mut is_shutdown = self.is_shutdown.write().await;
        *is_shutdown = true;

        // Cancel all running jobs
        Self::cleanup_all_jobs(Arc::clone(&self.running_jobs)).await;
    }

    /// Clean up completed job handles
    async fn cleanup_completed_jobs(running_jobs: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>) {
        let mut jobs = running_jobs.lock().await;
        let mut completed_jobs = Vec::new();

        for (job_id, handle) in jobs.iter() {
            if handle.is_finished() {
                completed_jobs.push(job_id.clone());
            }
        }

        for job_id in completed_jobs {
            jobs.remove(&job_id);
        }
    }

    /// Clean up all running jobs (for shutdown)
    async fn cleanup_all_jobs(running_jobs: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>) {
        let mut jobs = running_jobs.lock().await;
        
        for (_, handle) in jobs.drain() {
            handle.abort();
        }
    }

    /// Process all queued jobs (convenience method)
    pub async fn process_queued_jobs(&self) -> Result<(), String> {
        let job_ids = {
            let state_guard = self.state.read().await;
            state_guard.get_jobs_by_status(&JobStatus::Queued)
                .iter()
                .map(|job| job.id.clone())
                .collect::<Vec<_>>()
        };

        for job_id in job_ids {
            self.submit_job(job_id).await?;
        }

        Ok(())
    }

    /// Remove a job from the queue and clean up resources
    pub async fn remove_job(&self, job_id: &str) -> Result<(), String> {
        // First cancel the job if it's running
        self.cancel_job(job_id).await?;

        // Remove from state
        {
            let mut state_guard = self.state.write().await;
            if !state_guard.remove_job(job_id) {
                return Err("Job not found".to_string());
            }
        }

        Ok(())
    }

    /// Clear all completed and failed jobs
    pub async fn clear_completed_jobs(&self) -> Result<usize, String> {
        let mut state_guard = self.state.write().await;
        let initial_count = state_guard.jobs.len();
        state_guard.clear_completed_jobs();
        let final_count = state_guard.jobs.len();
        
        Ok(initial_count - final_count)
    }

    /// Cancel all jobs in the queue
    pub async fn cancel_all_jobs(&self) -> Result<usize, String> {
        let job_ids = {
            let state_guard = self.state.read().await;
            state_guard.jobs.iter()
                .filter(|job| matches!(job.status, JobStatus::Queued | JobStatus::Downloading))
                .map(|job| job.id.clone())
                .collect::<Vec<_>>()
        };

        let mut cancelled_count = 0;
        for job_id in job_ids {
            if self.cancel_job(&job_id).await.is_ok() {
                cancelled_count += 1;
            }
        }

        Ok(cancelled_count)
    }

    /// Retry all failed jobs
    pub async fn retry_all_failed_jobs(&self) -> Result<usize, String> {
        let failed_job_ids = {
            let state_guard = self.state.read().await;
            state_guard.get_jobs_by_status(&JobStatus::Failed)
                .iter()
                .map(|job| job.id.clone())
                .collect::<Vec<_>>()
        };

        let mut retried_count = 0;
        for job_id in failed_job_ids {
            if self.retry_job(job_id).await.is_ok() {
                retried_count += 1;
            }
        }

        Ok(retried_count)
    }

    /// Get detailed information about a specific job
    pub async fn get_job_info(&self, job_id: &str) -> Option<DownloadJob> {
        let state_guard = self.state.read().await;
        state_guard.get_job(job_id).cloned()
    }

    /// Update the concurrent limit for the queue
    pub async fn set_concurrent_limit(&mut self, limit: usize) -> Result<(), String> {
        if limit == 0 {
            return Err("Concurrent limit must be greater than 0".to_string());
        }

        self.concurrent_limit = limit;

        // Update the config in state as well
        {
            let mut state_guard = self.state.write().await;
            state_guard.config.concurrent_limit = limit;
        }

        Ok(())
    }

    /// Get the current concurrent limit
    pub fn get_concurrent_limit(&self) -> usize {
        self.concurrent_limit
    }

    /// Check if the queue manager is healthy (binary available, etc.)
    pub async fn health_check(&self) -> Result<String, String> {
        match self.gytmdl_wrapper.test_binary().await {
            Ok(version) => Ok(format!("Queue manager healthy. gytmdl version: {}", version)),
            Err(e) => Err(format!("Health check failed: {}", e)),
        }
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self) -> QueueStats {
        let state_guard = self.state.read().await;
        let running_count = self.running_jobs.lock().await.len();
        
        QueueStats {
            queued: state_guard.count_jobs_by_status(&JobStatus::Queued),
            downloading: running_count,
            completed: state_guard.count_jobs_by_status(&JobStatus::Completed),
            failed: state_guard.count_jobs_by_status(&JobStatus::Failed),
            cancelled: state_guard.count_jobs_by_status(&JobStatus::Cancelled),
            total: state_guard.jobs.len(),
            is_paused: *self.is_paused.read().await,
        }
    }

    /// Test the sidecar binary
    pub async fn test_sidecar(&self) -> Result<String, crate::modules::gytmdl_wrapper::GytmdlError> {
        self.gytmdl_wrapper.test_binary().await
    }

    /// Test download with dry run (no actual download)
    pub async fn test_download_dry_run(&self, url: &str) -> Result<String, crate::modules::gytmdl_wrapper::GytmdlError> {
        // Create a temporary job for testing
        let test_job = DownloadJob {
            id: "test-dry-run".to_string(),
            url: url.to_string(),
            status: JobStatus::Queued,
            progress: crate::modules::state::Progress {
                stage: crate::modules::state::DownloadStage::Initializing,
                percentage: None,
                current_step: "Testing".to_string(),
                total_steps: None,
                current_step_index: None,
            },
            metadata: None,
            error: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        // Get current config
        let config = {
            let state_guard = self.state.read().await;
            state_guard.config.clone()
        };

        // Test if we can build the command args (dry run)
        match self.gytmdl_wrapper.build_command_args(&config, url, &test_job.id) {
            Ok(args) => {
                let command_str = format!("gytmdl {:?}", args);
                Ok(format!("Dry run successful. Command would be: {}", command_str))
            }
            Err(e) => Err(e)
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub queued: usize,
    pub downloading: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub total: usize,
    pub is_paused: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::state::{AppState, DownloadJob, JobStatus};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::time::{sleep, Duration};

    async fn create_test_queue_manager() -> (QueueManager, Arc<RwLock<AppState>>) {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        // For testing, we'll create a mock queue manager that doesn't require gytmdl binary
        // We'll need to modify the constructor to accept an optional wrapper for testing
        match QueueManager::new(Arc::clone(&state), 2) {
            Ok(manager) => (manager, state),
            Err(_) => {
                // If gytmdl binary is not available, we'll skip these tests
                // In a real test environment, we'd use a mock wrapper
                panic!("gytmdl binary not available for testing");
            }
        }
    }

    #[tokio::test]
    async fn test_queue_manager_creation() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        // This test might fail if gytmdl binary is not available
        // In production tests, we'd use dependency injection for the wrapper
        if let Ok(manager) = QueueManager::new(Arc::clone(&state), 3) {
            assert_eq!(manager.get_concurrent_limit(), 3);
            assert!(!manager.is_paused().await);
            assert_eq!(manager.running_count().await, 0);
            assert_eq!(manager.queued_count().await, 0);
        }
    }

    #[tokio::test]
    async fn test_pause_resume_functionality() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        if let Ok(manager) = QueueManager::new(Arc::clone(&state), 2) {
            // Initially not paused
            assert!(!manager.is_paused().await);
            
            // Pause the queue
            manager.pause().await;
            assert!(manager.is_paused().await);
            
            // Resume the queue
            manager.resume().await;
            assert!(!manager.is_paused().await);
        }
    }

    #[tokio::test]
    async fn test_job_submission() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        if let Ok(manager) = QueueManager::new(Arc::clone(&state), 2) {
            // Add a job to state first
            let job_id = {
                let mut state_guard = state.write().await;
                state_guard.add_job("https://music.youtube.com/watch?v=test".to_string())
            };

            // Submit the job
            let result = manager.submit_job(job_id.clone()).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_calculate_backoff_delay() {
        // Test exponential backoff calculation
        assert_eq!(QueueManager::calculate_backoff_delay(0), 1000); // Base case (retry_count 0 -> 2^(-1) = 0.5, but saturating_sub makes it 0, so 2^0 = 1)
        assert_eq!(QueueManager::calculate_backoff_delay(1), 1000); // 1 second (2^0)
        assert_eq!(QueueManager::calculate_backoff_delay(2), 2000); // 2 seconds (2^1)
        assert_eq!(QueueManager::calculate_backoff_delay(3), 4000); // 4 seconds (2^2)
        assert_eq!(QueueManager::calculate_backoff_delay(4), 8000); // 8 seconds (2^3)
        
        // Test max delay cap
        let large_retry = QueueManager::calculate_backoff_delay(10);
        assert_eq!(large_retry, 30000); // Should be capped at 30 seconds
    }

    #[tokio::test]
    async fn test_concurrent_limit_update() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        if let Ok(mut manager) = QueueManager::new(Arc::clone(&state), 2) {
            assert_eq!(manager.get_concurrent_limit(), 2);
            
            // Update concurrent limit
            let result = manager.set_concurrent_limit(5).await;
            assert!(result.is_ok());
            assert_eq!(manager.get_concurrent_limit(), 5);
            
            // Test invalid limit
            let result = manager.set_concurrent_limit(0).await;
            assert!(result.is_err());
            assert_eq!(manager.get_concurrent_limit(), 5); // Should remain unchanged
        }
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        if let Ok(manager) = QueueManager::new(Arc::clone(&state), 2) {
            // Add some test jobs
            {
                let mut state_guard = state.write().await;
                let job_id1 = state_guard.add_job("https://test1.com".to_string());
                let job_id2 = state_guard.add_job("https://test2.com".to_string());
                let job_id3 = state_guard.add_job("https://test3.com".to_string());
                
                // Set different statuses
                state_guard.update_job_status(&job_id1, JobStatus::Downloading);
                state_guard.update_job_status(&job_id2, JobStatus::Completed);
                state_guard.update_job_status(&job_id3, JobStatus::Failed);
            }

            let stats = manager.get_queue_stats().await;
            assert_eq!(stats.total, 3);
            assert_eq!(stats.queued, 0);
            assert_eq!(stats.completed, 1);
            assert_eq!(stats.failed, 1);
            assert!(!stats.is_paused);
        }
    }

    #[tokio::test]
    async fn test_job_removal() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        if let Ok(manager) = QueueManager::new(Arc::clone(&state), 2) {
            // Add a job
            let job_id = {
                let mut state_guard = state.write().await;
                state_guard.add_job("https://test.com".to_string())
            };

            // Verify job exists
            assert!(manager.get_job_info(&job_id).await.is_some());

            // Remove the job
            let result = manager.remove_job(&job_id).await;
            assert!(result.is_ok());

            // Verify job is removed
            assert!(manager.get_job_info(&job_id).await.is_none());
        }
    }

    #[tokio::test]
    async fn test_clear_completed_jobs() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        if let Ok(manager) = QueueManager::new(Arc::clone(&state), 2) {
            // Add jobs with different statuses
            {
                let mut state_guard = state.write().await;
                let job_id1 = state_guard.add_job("https://test1.com".to_string());
                let job_id2 = state_guard.add_job("https://test2.com".to_string());
                let job_id3 = state_guard.add_job("https://test3.com".to_string());
                let job_id4 = state_guard.add_job("https://test4.com".to_string());
                
                state_guard.update_job_status(&job_id1, JobStatus::Completed);
                state_guard.update_job_status(&job_id2, JobStatus::Failed);
                state_guard.update_job_status(&job_id3, JobStatus::Downloading);
                // job_id4 remains Queued
            }

            let initial_stats = manager.get_queue_stats().await;
            assert_eq!(initial_stats.total, 4);

            // Clear completed jobs
            let cleared_count = manager.clear_completed_jobs().await.unwrap();
            assert_eq!(cleared_count, 2); // Should clear completed and failed

            let final_stats = manager.get_queue_stats().await;
            assert_eq!(final_stats.total, 2); // Only downloading and queued should remain
        }
    }

    #[tokio::test]
    async fn test_retry_job_validation() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        if let Ok(manager) = QueueManager::new(Arc::clone(&state), 2) {
            // Add a job and set it to failed status
            let job_id = {
                let mut state_guard = state.write().await;
                let id = state_guard.add_job("https://test.com".to_string());
                state_guard.update_job_status(&id, JobStatus::Failed);
                id
            };

            // Should be able to retry a failed job
            let result = manager.retry_job(job_id.clone()).await;
            assert!(result.is_ok());

            // Job should now be queued again
            let job_info = manager.get_job_info(&job_id).await.unwrap();
            assert_eq!(job_info.status, JobStatus::Queued);
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = Arc::new(RwLock::new(AppState::new()));
        
        if let Ok(manager) = QueueManager::new(Arc::clone(&state), 2) {
            // Health check should work if gytmdl binary is available
            let result = manager.health_check().await;
            // We can't guarantee the binary is available in test environment
            // so we just check that the method doesn't panic
            match result {
                Ok(msg) => assert!(msg.contains("healthy")),
                Err(msg) => assert!(msg.contains("Health check failed")),
            }
        }
    }
}