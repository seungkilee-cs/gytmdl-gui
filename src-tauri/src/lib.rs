pub mod modules;

use modules::state::{AppState, AppConfig};
use modules::config_manager::ConfigManager;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Placeholder commands - will be implemented in later tasks
#[tauri::command]
async fn get_queue(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<modules::state::DownloadJob>, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.jobs.clone())
}

#[tauri::command]
async fn add_to_queue(_url: String, _state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    // Placeholder - will be implemented in task 5
    Ok("placeholder".to_string())
}

fn get_state_file_path() -> PathBuf {
    // Use a simple approach for state file location
    let app_data_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    app_data_dir.join(".gytmdl-gui").join("state.json")
}

fn initialize_app_state() -> Arc<Mutex<AppState>> {
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
    
    Arc::new(Mutex::new(app_state))
}

#[tauri::command]
async fn save_state(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let state_file = get_state_file_path();
    
    app_state.save_to_file(&state_file)
        .map_err(|e| format!("Failed to save state: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn get_config(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<AppConfig, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.config.clone())
}

#[tauri::command]
async fn update_config(
    new_config: AppConfig,
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<(), String> {
    let config_manager = ConfigManager::with_default_path();
    
    // Validate the new config
    config_manager.validate_config(&new_config)
        .map_err(|e| format!("Configuration validation failed: {}", e))?;
    
    // Update the state
    {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.config = new_config.clone();
    }
    
    // Save the config to file
    config_manager.save_config(&new_config)
        .map_err(|e| format!("Failed to save configuration: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn reset_config_to_defaults(
    state: tauri::State<'_, Arc<Mutex<AppState>>>
) -> Result<AppConfig, String> {
    let config_manager = ConfigManager::with_default_path();
    let default_config = AppConfig::default();
    
    // Update the state
    {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.config = default_config.clone();
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = initialize_app_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet, 
            get_queue, 
            add_to_queue, 
            save_state,
            get_config,
            update_config,
            reset_config_to_defaults,
            validate_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
