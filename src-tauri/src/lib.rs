mod modules;

use modules::state::AppState;
use std::sync::{Arc, Mutex};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(Mutex::new(AppState::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![greet, get_queue, add_to_queue])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
