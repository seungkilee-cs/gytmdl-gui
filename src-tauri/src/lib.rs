pub mod modules;

use modules::state::{AppState, AppConfig, DownloadJob, JobStatus};
use modules::config_manager::ConfigManager;
use modules::queue_manager::QueueManager;
use modules::cookie_manager::CookieManager;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;

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

#[tauri::command]
async fn add_to_queue(url: String, context: tauri::State<'_, Arc<AppContext>>) -> Result<String, String> {
    // Validate URL format
    if url.trim().is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    // Basic URL validation - check if it's a valid HTTP/HTTPS URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL must start with http:// or https://".to_string());
    }

    // Check if it's a YouTube Music URL
    if !url.contains("music.youtube.com") && 
       !url.contains("youtube.com/watch") &&
       !url.contains("youtube.com/playlist") &&
       !url.contains("youtu.be/") {
        return Err("URL must be a valid YouTube Music URL".to_string());
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
            return Err(format!("Failed to submit job to queue: {}", e));
        }
    }

    Ok(job_id)
}

#[tauri::command]
async fn get_queue(context: tauri::State<'_, Arc<AppContext>>) -> Result<Vec<DownloadJob>, String> {
    let state_guard = context.state.read().await;
    Ok(state_guard.jobs.clone())
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

#[tauri::command]
async fn update_config(
    new_config: AppConfig,
    context: tauri::State<'_, Arc<AppContext>>
) -> Result<(), String> {
    let config_manager = ConfigManager::with_default_path();
    
    // Validate the new config
    config_manager.validate_config(&new_config)
        .map_err(|e| format!("Configuration validation failed: {}", e))?;
    
    // Update the state
    {
        let mut state_guard = context.state.write().await;
        state_guard.config = new_config.clone();
    }
    
    // Save the config to file
    config_manager.save_config(&new_config)
        .map_err(|e| format!("Failed to save configuration: {}", e))?;
    
    // Update queue manager concurrent limit if it changed
    if let Some(queue_manager) = context.queue_manager.read().await.as_ref() {
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

#[tauri::command]
async fn validate_config(config: AppConfig) -> Result<(), String> {
    let config_manager = ConfigManager::with_default_path();
    config_manager.validate_config(&config)
        .map_err(|e| format!("Configuration validation failed: {}", e))?;
    Ok(())
}

// Cookie Management Commands (Task 5.3)

#[tauri::command]
async fn import_cookies(file_path: String, context: tauri::State<'_, Arc<AppContext>>) -> Result<modules::cookie_manager::CookieInfo, String> {
    let cookie_manager = context.cookie_manager.read().await;
    let source_path = std::path::Path::new(&file_path);
    
    cookie_manager.import_cookies(source_path).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_cookies(context: tauri::State<'_, Arc<AppContext>>) -> Result<modules::cookie_manager::CookieInfo, String> {
    let cookie_manager = context.cookie_manager.read().await;
    
    cookie_manager.validate_cookies().await
        .map_err(|e| e.to_string())
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

    // Initialize queue manager in a separate task
    let context_for_init = Arc::clone(&app_context);
    tokio::spawn(async move {
        if let Err(e) = context_for_init.initialize_queue_manager().await {
            eprintln!("Failed to initialize queue manager: {}", e);
            eprintln!("Queue functionality will be limited until gytmdl binary is available");
        } else {
            println!("Queue manager initialized successfully");
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_context)
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
            // Utility Commands
            save_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
