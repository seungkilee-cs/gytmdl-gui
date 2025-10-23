pub mod modules;

use modules::state::{AppState, AppConfig, DownloadJob, JobStatus};
use modules::config_manager::ConfigManager;
use modules::queue_manager::QueueManager;
use modules::cookie_manager::CookieManager;
use modules::sidecar_manager::{get_sidecar_status, validate_sidecar_binaries, select_best_sidecar, check_sidecar_compatibility};
use modules::debug_logger::{DEBUG_LOGGER, LogEntry};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tauri::Manager;

/// Application context that holds shared state and managers
pub struct AppContext {
    pub state: Arc<RwLock<AppState>>,
    pub queue_manager: Arc<RwLock<Option<QueueManager>>>,
    pub cookie_manager: Arc<RwLock<CookieManager>>,
}

impl AppContext {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self {
            state: Arc::clone(&state),
            queue_manager: Arc::new(RwLock::new(None)),
            cookie_manager: Arc::new(RwLock::new(CookieManager::new())),
        }
    }

    pub async fn initialize_queue_manager(&self) -> Result<(), String> {
        let concurrent_limit = {
            let state_guard = self.state.read().await;
            state_guard.config.concurrent_limit
        };

        match QueueManager::new(Arc::clone(&self.state), concurrent_limit) {
            Ok(manager) => {
                // Start the queue manager
                if let Err(e) = manager.start().await {
                    return Err(format!("Failed to start queue manager: {}", e));
                }
                
                let mut queue_manager_guard = self.queue_manager.write().await;
                *queue_manager_guard = Some(manager);
                Ok(())
            }
            Err(e) => Err(format!("Failed to create queue manager: {}", e))
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Queue Management Commands (Task 5.1)

#[derive(serde::Serialize)]
struct AddJobResponse {
    success: bool,
    job_id: Option<String>,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct AddJobRequest {
    url: String,
}

#[tauri::command]
async fn add_to_queue(request: AddJobRequest, context: tauri::State<'_, Arc<AppContext>>) -> Result<AddJobResponse, String> {
    let url = request.url;
    
    // Validate URL format
    if url.trim().is_empty() {
        return Ok(AddJobResponse {
            success: false,
            job_id: None,
            error: Some("URL cannot be empty".to_string()),
        });
    }

    // Basic URL validation - check if it's a valid HTTP/HTTPS URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(AddJobResponse {
            success: false,
            job_id: None,
            error: Some("URL must start with http:// or https://".to_string()),
        });
    }

    // Check if it's a YouTube Music URL
    if !url.contains("music.youtube.com") && 
       !url.contains("youtube.com/watch") &&
       !url.contains("youtube.com/playlist") &&
       !url.contains("youtu.be/") {
        return Ok(AddJobResponse {
            success: false,
            job_id: None,
            error: Some("URL must be a valid YouTube Music URL".to_string()),
        });
    }

    // Add job to state
    let job_id = {
        let mut state_guard = context.state.write().await;
        state_guard.add_job(url)
    };

    // Submit job to queue manager if available
    if let Some(queue_manager) = context.queue_manager.read().await.as_ref() {
        if let Err(e) = queue_manager.submit_job(job_id.clone()).await {
            // If submission fails, remove the job from state
            let mut state_guard = context.state.write().await;
            state_guard.remove_job(&job_id);
            return Ok(AddJobResponse {
                success: false,
                job_id: None,
                error: Some(format!("Failed to submit job to queue: {}", e)),
            });
        }
    }

    Ok(AddJobResponse {
        success: true,
        job_id: Some(job_id),
        error: None,
    })
}

#[derive(serde::Serialize)]
struct QueueState {
    jobs: Vec<DownloadJob>,
    is_paused: bool,
    concurrent_limit: usize,
}

#[tauri::command]
async fn get_queue(context: tauri::State<'_, Arc<AppContext>>) -> Result<QueueState, String> {
    let state_guard = context.state.read().await;
    Ok(QueueState {
        jobs: state_guard.jobs.clone(),
        is_paused: state_guard.is_paused,
        concurrent_limit: state_guard.config.concurrent_limit,
    })
}

#[tauri::command]
async fn retry_job(job_id: String, context: tauri::State<'_, Arc<AppContext>>) -> Result<(), String> {
    // Check if job exists and can be retried
    {
        let state_guard = context.state.read().await;
        if let Some(job) = state_guard.get_job(&job_id) {
            if !job.can_retry() {
                return Err("Job cannot be retried".to_string());
            }
        } else {
            return Err("Job not found".to_string());
        }
    }

    // Retry job using queue manager
    if let Some(queue_manager) = context.queue_manager.read().await.as_ref() {
        queue_manager.retry_job(job_id).await
    } else {
        Err("Queue manager not available".to_string())
    }
}

#[tauri::command]
async fn cancel_job(job_id: String, context: tauri::State<'_, Arc<AppContext>>) -> Result<(), String> {
    // Check if job exists
    {
        let state_guard = context.state.read().await;
        if state_guard.get_job(&job_id).is_none() {
            return Err("Job not found".to_string());
        }
    }

    // Cancel job using queue manager
    if let Some(queue_manager) = context.queue_manager.read().await.as_ref() {
        queue_manager.cancel_job(&job_id).await
    } else {
        // If queue manager not available, just update state
        let mut state_guard = context.state.write().await;
        state_guard.update_job_status(&job_id, JobStatus::Cancelled);
        Ok(())
    }
}

#[tauri::command]
async fn pause_queue(context: tauri::State<'_, Arc<AppContext>>) -> Result<(), String> {
    if let Some(queue_manager) = context.queue_manager.read().await.as_ref() {
        queue_manager.pause().await;
        Ok(())
    } else {
        // If queue manager not available, just update state
        let mut state_guard = context.state.write().await;
        state_guard.pause();
        Ok(())
    }
}

#[tauri::command]
async fn resume_queue(context: tauri::State<'_, Arc<AppContext>>) -> Result<(), String> {
    if let Some(queue_manager) = context.queue_manager.read().await.as_ref() {
        queue_manager.resume().await;
        Ok(())
    } else {
        // If queue manager not available, just update state
        let mut state_guard = context.state.write().await;
        state_guard.resume();
        Ok(())
    }
}

fn get_state_file_path() -> PathBuf {
    // Use a simple approach for state file location
    let app_data_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    app_data_dir.join(".gytmdl-gui").join("state.json")
}

fn initialize_app_state() -> Arc<RwLock<AppState>> {
    let state_file = get_state_file_path();
    let config_manager = ConfigManager::with_default_path();
    
    // Try to load existing state, fallback to default if it fails
    let mut app_state = match AppState::load_from_file(&state_file) {
        Ok(state) => {
            println!("Loaded existing state from: {:?}", state_file);
            state
        }
        Err(e) => {
            println!("Failed to load state from {:?}: {}. Using default state.", state_file, e);
            AppState::default()
        }
    };
    
    // Load configuration separately and update state
    match config_manager.load_config() {
        Ok(config) => {
            println!("Loaded configuration from: {:?}", config_manager.get_config_file_path());
            app_state.config = config;
        }
        Err(e) => {
            println!("Failed to load config: {}. Using default config.", e);
            app_state.config = AppConfig::default();
            // Try to save the default config
            if let Err(save_err) = config_manager.save_config(&app_state.config) {
                println!("Failed to save default config: {}", save_err);
            }
        }
    }
    
    Arc::new(RwLock::new(app_state))
}

#[tauri::command]
async fn remove_job(job_id: String, context: tauri::State<'_, Arc<AppContext>>) -> Result<(), String> {
    // Check if job exists
    {
        let state_guard = context.state.read().await;
        if state_guard.get_job(&job_id).is_none() {
            return Err("Job not found".to_string());
        }
    }

    // Remove job from state
    let mut state_guard = context.state.write().await;
    state_guard.remove_job(&job_id);
    Ok(())
}

#[tauri::command]
async fn clear_completed_jobs(context: tauri::State<'_, Arc<AppContext>>) -> Result<(), String> {
    let mut state_guard = context.state.write().await;
    state_guard.jobs.retain(|job| job.status != JobStatus::Completed);
    Ok(())
}

#[tauri::command]
async fn save_state(context: tauri::State<'_, Arc<AppContext>>) -> Result<(), String> {
    let state_guard = context.state.read().await;
    let state_file = get_state_file_path();
    
    state_guard.save_to_file(&state_file)
        .map_err(|e| format!("Failed to save state: {}", e))?;
    
    Ok(())
}

// Configuration Management Commands (Task 5.2)

#[tauri::command]
async fn get_config(context: tauri::State<'_, Arc<AppContext>>) -> Result<AppConfig, String> {
    let state_guard = context.state.read().await;
    Ok(state_guard.config.clone())
}

#[derive(serde::Deserialize)]
struct UpdateConfigRequest {
    config: AppConfig,
}

#[tauri::command]
async fn update_config(
    request: UpdateConfigRequest,
    context: tauri::State<'_, Arc<AppContext>>
) -> Result<(), String> {
    let config_manager = ConfigManager::with_default_path();
    
    // Validate the new config
    config_manager.validate_config(&request.config)
        .map_err(|e| format!("Configuration validation failed: {}", e))?;
    
    // Update the state
    {
        let mut state_guard = context.state.write().await;
        state_guard.config = request.config.clone();
    }
    
    // Save the config to file
    config_manager.save_config(&request.config)
        .map_err(|e| format!("Failed to save configuration: {}", e))?;
    
    // Update queue manager concurrent limit if it changed
    if let Some(_queue_manager) = context.queue_manager.read().await.as_ref() {
        // Note: QueueManager::set_concurrent_limit requires &mut self, 
        // so we'd need to restructure this or add a method that works with Arc<RwLock<>>
        // For now, we'll just log that the limit should be updated on next restart
        println!("Configuration updated. Queue manager concurrent limit will be updated on next restart.");
    }
    
    Ok(())
}

#[tauri::command]
async fn reset_config_to_defaults(
    context: tauri::State<'_, Arc<AppContext>>
) -> Result<AppConfig, String> {
    let config_manager = ConfigManager::with_default_path();
    let default_config = AppConfig::default();
    
    // Update the state
    {
        let mut state_guard = context.state.write().await;
        state_guard.config = default_config.clone();
    }
    
    // Save the default config to file
    config_manager.save_config(&default_config)
        .map_err(|e| format!("Failed to save default configuration: {}", e))?;
    
    Ok(default_config)
}

#[derive(serde::Serialize)]
struct ConfigValidationError {
    field: String,
    message: String,
}

#[derive(serde::Serialize)]
struct ConfigValidationResult {
    #[serde(rename = "isValid")]
    is_valid: bool,
    errors: Vec<ConfigValidationError>,
}

#[derive(serde::Deserialize)]
struct ValidateConfigRequest {
    config: AppConfig,
}

#[tauri::command]
async fn validate_config(request: ValidateConfigRequest) -> Result<ConfigValidationResult, String> {
    let config_manager = ConfigManager::with_default_path();
    
    match config_manager.validate_config(&request.config) {
        Ok(()) => Ok(ConfigValidationResult {
            is_valid: true,
            errors: vec![],
        }),
        Err(e) => Ok(ConfigValidationResult {
            is_valid: false,
            errors: vec![ConfigValidationError {
                field: "general".to_string(),
                message: e.to_string(),
            }],
        })
    }
}

// Cookie Management Commands (Task 5.3)

#[derive(serde::Serialize)]
struct CookieImportResult {
    success: bool,
    cookies_count: Option<usize>,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct CookieImportRequest {
    file_path: String,
}

#[tauri::command]
async fn import_cookies(request: CookieImportRequest, context: tauri::State<'_, Arc<AppContext>>) -> Result<CookieImportResult, String> {
    let cookie_manager = context.cookie_manager.read().await;
    let source_path = std::path::Path::new(&request.file_path);
    
    match cookie_manager.import_cookies(source_path).await {
        Ok(_cookie_info) => Ok(CookieImportResult {
            success: true,
            cookies_count: Some(1), // We don't track individual cookie count, just indicate success
            error: None,
        }),
        Err(e) => Ok(CookieImportResult {
            success: false,
            cookies_count: None,
            error: Some(e.to_string()),
        })
    }
}

#[derive(serde::Serialize)]
struct CookieValidationResult {
    is_valid: bool,
    expiration_date: Option<String>,
    days_until_expiry: Option<i64>,
    has_po_token: bool,
    error: Option<String>,
}

#[tauri::command]
async fn validate_cookies(context: tauri::State<'_, Arc<AppContext>>) -> Result<CookieValidationResult, String> {
    let cookie_manager = context.cookie_manager.read().await;
    
    match cookie_manager.validate_cookies().await {
        Ok(cookie_info) => {
            // Extract days until expiry from expiration_warning if available
            let days_until_expiry = if let Some(ref warning) = cookie_info.expiration_warning {
                // Try to extract number of days from warning message
                if warning.contains("expires in") {
                    warning.split("expires in ")
                        .nth(1)
                        .and_then(|s| s.split(" days").next())
                        .and_then(|s| s.parse::<i64>().ok())
                } else {
                    None
                }
            } else {
                None
            };

            Ok(CookieValidationResult {
                is_valid: cookie_info.is_valid,
                expiration_date: cookie_info.expiration_warning.clone(),
                days_until_expiry,
                has_po_token: cookie_info.po_token_present,
                error: None,
            })
        },
        Err(e) => Ok(CookieValidationResult {
            is_valid: false,
            expiration_date: None,
            days_until_expiry: None,
            has_po_token: false,
            error: Some(e.to_string()),
        })
    }
}

#[tauri::command]
async fn get_cookies_path(context: tauri::State<'_, Arc<AppContext>>) -> Result<String, String> {
    let cookie_manager = context.cookie_manager.read().await;
    let path = cookie_manager.get_cookies_path();
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn clear_cookies(context: tauri::State<'_, Arc<AppContext>>) -> Result<(), String> {
    let cookie_manager = context.cookie_manager.read().await;
    
    cookie_manager.clear_cookies().await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = initialize_app_state();
    let app_context = Arc::new(AppContext::new(app_state));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_context)
        .setup(|app| {
            // Initialize queue manager after Tauri runtime is available
            let app_context = app.state::<Arc<AppContext>>();
            let context_for_init: Arc<AppContext> = Arc::clone(app_context.inner());
            
            tauri::async_runtime::spawn(async move {
                if let Err(e) = context_for_init.initialize_queue_manager().await {
                    eprintln!("Failed to initialize queue manager: {}", e);
                    eprintln!("Queue functionality will be limited until gytmdl binary is available");
                } else {
                    println!("Queue manager initialized successfully");
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // Queue Management Commands
            add_to_queue,
            get_queue, 
            retry_job,
            cancel_job,
            pause_queue,
            resume_queue,
            // Configuration Management Commands
            get_config,
            update_config,
            reset_config_to_defaults,
            validate_config,
            // Cookie Management Commands
            import_cookies,
            validate_cookies,
            get_cookies_path,
            clear_cookies,
            // Additional Queue Commands
            remove_job,
            clear_completed_jobs,
            // Utility Commands
            save_state,
            // Sidecar Management Commands
            get_sidecar_status,
            validate_sidecar_binaries,
            select_best_sidecar,
            check_sidecar_compatibility,
            // Debug Commands
            get_debug_logs,
            clear_debug_logs,
            test_sidecar_binary,
            test_download_dry_run
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
