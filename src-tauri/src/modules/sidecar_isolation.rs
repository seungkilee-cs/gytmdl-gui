use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use tauri::Manager;

/// Configuration isolation manager for gytmdl sidecar
/// Ensures the GUI app's sidecar doesn't interfere with system gytmdl installation
#[derive(Debug, Clone)]
pub struct SidecarIsolation {
    /// Isolated config directory for the GUI app
    config_dir: PathBuf,
    /// Isolated cache directory for the GUI app
    cache_dir: PathBuf,
    /// Isolated data directory for the GUI app
    data_dir: PathBuf,
    /// Environment variables to set for sidecar execution
    env_vars: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IsolationConfig {
    /// Whether to use isolated directories
    pub use_isolation: bool,
    /// Custom config directory path (optional)
    pub custom_config_dir: Option<PathBuf>,
    /// Custom cache directory path (optional)
    pub custom_cache_dir: Option<PathBuf>,
    /// Custom data directory path (optional)
    pub custom_data_dir: Option<PathBuf>,
    /// Additional environment variables
    pub additional_env_vars: HashMap<String, String>,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            use_isolation: true,
            custom_config_dir: None,
            custom_cache_dir: None,
            custom_data_dir: None,
            additional_env_vars: HashMap::new(),
        }
    }
}

impl SidecarIsolation {
    /// Create a new sidecar isolation manager
    pub fn new(config: IsolationConfig) -> Result<Self, String> {
        if !config.use_isolation {
            return Ok(Self {
                config_dir: PathBuf::new(),
                cache_dir: PathBuf::new(),
                data_dir: PathBuf::new(),
                env_vars: config.additional_env_vars,
            });
        }

        let app_data_dir = Self::get_app_data_dir()?;
        
        let config_dir = config.custom_config_dir
            .unwrap_or_else(|| app_data_dir.join("config"));
        
        let cache_dir = config.custom_cache_dir
            .unwrap_or_else(|| app_data_dir.join("cache"));
        
        let data_dir = config.custom_data_dir
            .unwrap_or_else(|| app_data_dir.join("data"));

        // Create directories if they don't exist
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let mut env_vars = Self::create_isolation_env_vars(&config_dir, &cache_dir, &data_dir)?;
        
        // Add any additional environment variables
        for (key, value) in config.additional_env_vars {
            env_vars.insert(key, value);
        }

        Ok(Self {
            config_dir,
            cache_dir,
            data_dir,
            env_vars,
        })
    }

    /// Get the application data directory
    fn get_app_data_dir() -> Result<PathBuf, String> {
        // Try to get the app-specific data directory
        // Note: In Tauri v2, we'll use platform-specific paths directly

        // Fallback to platform-specific directories
        let base_dir = match std::env::consts::OS {
            "windows" => {
                env::var("APPDATA")
                    .map(PathBuf::from)
                    .or_else(|_| env::var("USERPROFILE").map(|p| PathBuf::from(p).join("AppData").join("Roaming")))
                    .map_err(|_| "Could not determine Windows app data directory")?
            }
            "macos" => {
                env::var("HOME")
                    .map(|p| PathBuf::from(p).join("Library").join("Application Support"))
                    .map_err(|_| "Could not determine macOS app data directory")?
            }
            "linux" => {
                env::var("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .or_else(|_| env::var("HOME").map(|p| PathBuf::from(p).join(".config")))
                    .map_err(|_| "Could not determine Linux config directory")?
            }
            _ => {
                return Err("Unsupported operating system".to_string());
            }
        };

        Ok(base_dir.join("gytmdl-gui"))
    }

    /// Create environment variables for sidecar isolation
    fn create_isolation_env_vars(
        config_dir: &Path,
        cache_dir: &Path,
        data_dir: &Path,
    ) -> Result<HashMap<String, String>, String> {
        let mut env_vars = HashMap::new();

        // Set gytmdl-specific environment variables to use isolated directories
        env_vars.insert(
            "GYTMDL_CONFIG_DIR".to_string(),
            config_dir.to_string_lossy().to_string(),
        );
        
        env_vars.insert(
            "GYTMDL_CACHE_DIR".to_string(),
            cache_dir.to_string_lossy().to_string(),
        );
        
        env_vars.insert(
            "GYTMDL_DATA_DIR".to_string(),
            data_dir.to_string_lossy().to_string(),
        );

        // Platform-specific environment variables
        match std::env::consts::OS {
            "windows" => {
                // Windows-specific isolation
                env_vars.insert(
                    "APPDATA".to_string(),
                    data_dir.to_string_lossy().to_string(),
                );
                env_vars.insert(
                    "LOCALAPPDATA".to_string(),
                    cache_dir.to_string_lossy().to_string(),
                );
            }
            "macos" | "linux" => {
                // Unix-like systems
                env_vars.insert(
                    "XDG_CONFIG_HOME".to_string(),
                    config_dir.to_string_lossy().to_string(),
                );
                env_vars.insert(
                    "XDG_CACHE_HOME".to_string(),
                    cache_dir.to_string_lossy().to_string(),
                );
                env_vars.insert(
                    "XDG_DATA_HOME".to_string(),
                    data_dir.to_string_lossy().to_string(),
                );
            }
            _ => {}
        }

        // Prevent gytmdl from using system-wide config
        env_vars.insert("GYTMDL_NO_SYSTEM_CONFIG".to_string(), "1".to_string());
        
        // Set a unique identifier for GUI app usage
        env_vars.insert("GYTMDL_GUI_MODE".to_string(), "1".to_string());

        Ok(env_vars)
    }

    /// Apply isolation environment to a command
    pub fn apply_to_command<'a>(&self, command: &'a mut Command) -> &'a mut Command {
        for (key, value) in &self.env_vars {
            command.env(key, value);
        }
        command
    }

    /// Get the isolated config directory
    pub fn get_config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get the isolated cache directory
    pub fn get_cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get the isolated data directory
    pub fn get_data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Get all isolation environment variables
    pub fn get_env_vars(&self) -> &HashMap<String, String> {
        &self.env_vars
    }

    /// Create a gytmdl config file in the isolated directory
    pub fn create_isolated_config(&self, config_content: &str) -> Result<PathBuf, String> {
        let config_file = self.config_dir.join("gytmdl.conf");
        
        std::fs::write(&config_file, config_content)
            .map_err(|e| format!("Failed to write isolated config: {}", e))?;
        
        Ok(config_file)
    }

    /// Check if system gytmdl is installed and get its version
    pub fn check_system_gytmdl(&self) -> Result<Option<String>, String> {
        // Create a command without isolation to check system gytmdl
        let output = Command::new("gytmdl")
            .arg("--version")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(Some(version))
            }
            Ok(_) => Ok(None), // gytmdl exists but --version failed
            Err(_) => Ok(None), // gytmdl not found
        }
    }

    /// Get information about the isolation setup
    pub fn get_isolation_info(&self) -> IsolationInfo {
        IsolationInfo {
            is_isolated: !self.config_dir.as_os_str().is_empty(),
            config_dir: self.config_dir.clone(),
            cache_dir: self.cache_dir.clone(),
            data_dir: self.data_dir.clone(),
            env_vars_count: self.env_vars.len(),
            system_gytmdl_version: self.check_system_gytmdl().unwrap_or(None),
        }
    }

    /// Migrate existing gytmdl config to isolated directory (if user wants)
    pub fn migrate_system_config(&self, force: bool) -> Result<bool, String> {
        if self.config_dir.as_os_str().is_empty() {
            return Ok(false); // No isolation, nothing to migrate
        }

        // Try to find system gytmdl config
        let system_config_paths = self.get_system_config_paths();
        
        for system_config_path in system_config_paths {
            if system_config_path.exists() {
                let isolated_config_path = self.config_dir.join("gytmdl.conf");
                
                if isolated_config_path.exists() && !force {
                    // Config already exists, don't overwrite unless forced
                    continue;
                }

                // Copy system config to isolated directory
                std::fs::copy(&system_config_path, &isolated_config_path)
                    .map_err(|e| format!("Failed to migrate config from {:?}: {}", system_config_path, e))?;
                
                return Ok(true);
            }
        }

        Ok(false) // No system config found to migrate
    }

    /// Get potential system gytmdl config file paths
    fn get_system_config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        match std::env::consts::OS {
            "windows" => {
                if let Ok(appdata) = env::var("APPDATA") {
                    paths.push(PathBuf::from(appdata).join("gytmdl").join("gytmdl.conf"));
                }
                if let Ok(userprofile) = env::var("USERPROFILE") {
                    paths.push(PathBuf::from(userprofile).join(".gytmdl").join("gytmdl.conf"));
                }
            }
            "macos" => {
                if let Ok(home) = env::var("HOME") {
                    paths.push(PathBuf::from(&home).join("Library").join("Application Support").join("gytmdl").join("gytmdl.conf"));
                    paths.push(PathBuf::from(&home).join(".gytmdl").join("gytmdl.conf"));
                }
            }
            "linux" => {
                if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
                    paths.push(PathBuf::from(xdg_config).join("gytmdl").join("gytmdl.conf"));
                }
                if let Ok(home) = env::var("HOME") {
                    paths.push(PathBuf::from(&home).join(".config").join("gytmdl").join("gytmdl.conf"));
                    paths.push(PathBuf::from(&home).join(".gytmdl").join("gytmdl.conf"));
                }
            }
            _ => {}
        }

        paths
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IsolationInfo {
    pub is_isolated: bool,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
    pub env_vars_count: usize,
    pub system_gytmdl_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_isolation_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = IsolationConfig {
            use_isolation: true,
            custom_config_dir: Some(temp_dir.path().join("config")),
            custom_cache_dir: Some(temp_dir.path().join("cache")),
            custom_data_dir: Some(temp_dir.path().join("data")),
            additional_env_vars: HashMap::new(),
        };

        let isolation = SidecarIsolation::new(config).unwrap();
        
        assert!(isolation.get_config_dir().exists());
        assert!(isolation.get_cache_dir().exists());
        assert!(isolation.get_data_dir().exists());
        assert!(!isolation.get_env_vars().is_empty());
    }

    #[test]
    fn test_no_isolation() {
        let config = IsolationConfig {
            use_isolation: false,
            ..Default::default()
        };

        let isolation = SidecarIsolation::new(config).unwrap();
        
        assert!(isolation.get_config_dir().as_os_str().is_empty());
        assert!(isolation.get_cache_dir().as_os_str().is_empty());
        assert!(isolation.get_data_dir().as_os_str().is_empty());
    }

    #[test]
    fn test_command_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let config = IsolationConfig {
            use_isolation: true,
            custom_config_dir: Some(temp_dir.path().join("config")),
            custom_cache_dir: Some(temp_dir.path().join("cache")),
            custom_data_dir: Some(temp_dir.path().join("data")),
            additional_env_vars: {
                let mut vars = HashMap::new();
                vars.insert("TEST_VAR".to_string(), "test_value".to_string());
                vars
            },
        };

        let isolation = SidecarIsolation::new(config).unwrap();
        let mut command = Command::new("echo");
        
        isolation.apply_to_command(&mut command);
        
        // The command should now have isolation environment variables set
        // This is hard to test directly, but we can verify the isolation has the right env vars
        assert!(isolation.get_env_vars().contains_key("GYTMDL_CONFIG_DIR"));
        assert!(isolation.get_env_vars().contains_key("TEST_VAR"));
    }
}