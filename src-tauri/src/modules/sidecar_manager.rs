use crate::modules::gytmdl_wrapper::{GytmdlWrapper, BinaryManifest};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarInfo {
    pub binary_path: String,
    pub is_available: bool,
    pub is_valid: bool,
    pub version: Option<String>,
    pub manifest: Option<BinaryManifest>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarStatus {
    pub current_binary: Option<SidecarInfo>,
    pub available_binaries: Vec<SidecarInfo>,
    pub platform_binary_name: String,
    pub sidecar_directory: String,
}

pub struct SidecarManager;

impl SidecarManager {
    /// Get comprehensive status of all sidecar binaries
    pub async fn get_status() -> SidecarStatus {
        let platform_binary_name = GytmdlWrapper::get_platform_binary_name();
        let sidecar_directory = GytmdlWrapper::get_sidecar_directory();
        
        // Try to get current binary info
        let current_binary = match GytmdlWrapper::new() {
            Ok(wrapper) => Some(Self::get_binary_info(&wrapper).await),
            Err(_) => None,
        };

        // Get all available binaries
        let available_binaries = match GytmdlWrapper::list_available_binaries() {
            Ok(binaries) => {
                let mut binary_infos = Vec::new();
                for binary_path in binaries {
                    if let Ok(wrapper) = GytmdlWrapper::with_binary_path(binary_path) {
                        binary_infos.push(Self::get_binary_info(&wrapper).await);
                    }
                }
                binary_infos
            }
            Err(_) => Vec::new(),
        };

        SidecarStatus {
            current_binary,
            available_binaries,
            platform_binary_name,
            sidecar_directory: sidecar_directory.to_string_lossy().to_string(),
        }
    }

    /// Get detailed information about a specific binary
    async fn get_binary_info(wrapper: &GytmdlWrapper) -> SidecarInfo {
        let binary_path = wrapper.get_binary_path().to_string_lossy().to_string();
        let is_available = wrapper.is_binary_available();
        
        let mut info = SidecarInfo {
            binary_path,
            is_available,
            is_valid: false,
            version: None,
            manifest: None,
            error: None,
        };

        if !is_available {
            info.error = Some("Binary file not found or not accessible".to_string());
            return info;
        }

        // Test binary functionality
        match wrapper.test_binary().await {
            Ok(version) => {
                info.version = Some(version);
                info.is_valid = true;
            }
            Err(e) => {
                info.error = Some(format!("Binary test failed: {}", e));
                return info;
            }
        }

        // Load manifest if available
        match wrapper.load_manifest() {
            Ok(manifest) => {
                info.manifest = Some(manifest);
            }
            Err(e) => {
                // Manifest is optional, so we don't fail here
                info.error = Some(format!("Manifest not available: {}", e));
            }
        }

        // Validate integrity if manifest is available
        if info.manifest.is_some() {
            match wrapper.validate_integrity() {
                Ok(true) => {
                    // Integrity check passed
                }
                Ok(false) => {
                    info.is_valid = false;
                    info.error = Some("Binary integrity validation failed".to_string());
                }
                Err(e) => {
                    info.error = Some(format!("Integrity check error: {}", e));
                }
            }
        }

        info
    }

    /// Validate all available binaries
    pub async fn validate_all_binaries() -> Result<Vec<SidecarInfo>, String> {
        let available_binaries = GytmdlWrapper::list_available_binaries()
            .map_err(|e| format!("Failed to list binaries: {}", e))?;

        let mut results = Vec::new();
        
        for binary_path in available_binaries {
            match GytmdlWrapper::with_binary_path(binary_path) {
                Ok(wrapper) => {
                    results.push(Self::get_binary_info(&wrapper).await);
                }
                Err(e) => {
                    results.push(SidecarInfo {
                        binary_path: "unknown".to_string(),
                        is_available: false,
                        is_valid: false,
                        version: None,
                        manifest: None,
                        error: Some(format!("Failed to create wrapper: {}", e)),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Select and validate the best available binary
    pub async fn select_best_binary() -> Result<SidecarInfo, String> {
        let binary_path = GytmdlWrapper::select_best_binary()
            .map_err(|e| format!("Failed to select binary: {}", e))?;

        let wrapper = GytmdlWrapper::with_binary_path(binary_path)
            .map_err(|e| format!("Failed to create wrapper: {}", e))?;

        Ok(Self::get_binary_info(&wrapper).await)
    }

    /// Check if the current platform has a suitable binary
    pub async fn check_platform_compatibility() -> Result<bool, String> {
        let status = Self::get_status().await;
        
        // Check if we have any valid binary
        if let Some(current) = &status.current_binary {
            if current.is_valid {
                return Ok(true);
            }
        }

        // Check if any available binary is valid
        for binary in &status.available_binaries {
            if binary.is_valid {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

// Tauri commands for sidecar management
#[tauri::command]
pub async fn get_sidecar_status() -> Result<SidecarStatus, String> {
    Ok(SidecarManager::get_status().await)
}

#[tauri::command]
pub async fn validate_sidecar_binaries() -> Result<Vec<SidecarInfo>, String> {
    SidecarManager::validate_all_binaries().await
}

#[tauri::command]
pub async fn select_best_sidecar() -> Result<SidecarInfo, String> {
    SidecarManager::select_best_binary().await
}

#[tauri::command]
pub async fn check_sidecar_compatibility() -> Result<bool, String> {
    SidecarManager::check_platform_compatibility().await
}