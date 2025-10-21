use crate::modules::state::AppConfig;
use serde_json;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    SerializationError(serde_json::Error),
    ValidationError(String),
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError::SerializationError(error)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ConfigError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

pub struct ConfigManager {
    config_file_path: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager with the specified config file path
    pub fn new(config_file_path: PathBuf) -> Self {
        Self { config_file_path }
    }

    /// Create a ConfigManager with default config file path
    pub fn with_default_path() -> Self {
        let config_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".gytmdl-gui");
        Self::new(config_dir.join("config.json"))
    }

    /// Load configuration from file, return default if file doesn't exist
    pub fn load_config(&self) -> Result<AppConfig, ConfigError> {
        if !self.config_file_path.exists() {
            // Return default config if file doesn't exist
            return Ok(AppConfig::default());
        }

        let content = fs::read_to_string(&self.config_file_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        
        // Validate the loaded config
        self.validate_config(&config)?;
        
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_config(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // Validate config before saving
        self.validate_config(config)?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_file_path, content)?;
        
        Ok(())
    }

    /// Validate configuration values
    pub fn validate_config(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // Validate paths exist or can be created
        if !config.output_path.exists() {
            if let Err(e) = fs::create_dir_all(&config.output_path) {
                return Err(ConfigError::ValidationError(
                    format!("Cannot create output directory {:?}: {}", config.output_path, e)
                ));
            }
        }

        if !config.temp_path.exists() {
            if let Err(e) = fs::create_dir_all(&config.temp_path) {
                return Err(ConfigError::ValidationError(
                    format!("Cannot create temp directory {:?}: {}", config.temp_path, e)
                ));
            }
        }

        // Validate cookies path if provided
        if let Some(cookies_path) = &config.cookies_path {
            if !cookies_path.exists() {
                return Err(ConfigError::ValidationError(
                    format!("Cookies file does not exist: {:?}", cookies_path)
                ));
            }
        }

        // Validate itag format (should be numeric)
        if config.itag.parse::<u32>().is_err() {
            return Err(ConfigError::ValidationError(
                format!("Invalid itag format: '{}'. Must be a number.", config.itag)
            ));
        }

        // Validate concurrent limit
        if config.concurrent_limit == 0 {
            return Err(ConfigError::ValidationError(
                "Concurrent limit must be greater than 0".to_string()
            ));
        }

        if config.concurrent_limit > 10 {
            return Err(ConfigError::ValidationError(
                "Concurrent limit should not exceed 10 for stability".to_string()
            ));
        }

        // Validate cover settings
        if config.cover_size == 0 {
            return Err(ConfigError::ValidationError(
                "Cover size must be greater than 0".to_string()
            ));
        }

        if config.cover_quality == 0 || config.cover_quality > 100 {
            return Err(ConfigError::ValidationError(
                "Cover quality must be between 1 and 100".to_string()
            ));
        }

        // Validate truncate value if provided
        if let Some(truncate) = config.truncate {
            if truncate == 0 {
                return Err(ConfigError::ValidationError(
                    "Truncate value must be greater than 0 if specified".to_string()
                ));
            }
        }

        // Validate template strings are not empty
        if config.template_folder.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Template folder cannot be empty".to_string()
            ));
        }

        if config.template_file.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Template file cannot be empty".to_string()
            ));
        }

        Ok(())
    }

    /// Update specific configuration values with validation
    pub fn update_config(&self, current_config: &mut AppConfig, updates: AppConfig) -> Result<(), ConfigError> {
        // Create a temporary config with updates applied
        let mut new_config = current_config.clone();
        
        // Apply updates
        new_config.output_path = updates.output_path;
        new_config.temp_path = updates.temp_path;
        new_config.cookies_path = updates.cookies_path;
        new_config.itag = updates.itag;
        new_config.download_mode = updates.download_mode;
        new_config.concurrent_limit = updates.concurrent_limit;
        new_config.cover_size = updates.cover_size;
        new_config.cover_format = updates.cover_format;
        new_config.cover_quality = updates.cover_quality;
        new_config.template_folder = updates.template_folder;
        new_config.template_file = updates.template_file;
        new_config.template_date = updates.template_date;
        new_config.po_token = updates.po_token;
        new_config.exclude_tags = updates.exclude_tags;
        new_config.truncate = updates.truncate;
        new_config.save_cover = updates.save_cover;
        new_config.overwrite = updates.overwrite;
        new_config.no_synced_lyrics = updates.no_synced_lyrics;

        // Validate the new config
        self.validate_config(&new_config)?;

        // If validation passes, apply the changes
        *current_config = new_config;
        
        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset_to_defaults(&self) -> AppConfig {
        AppConfig::default()
    }

    /// Get the config file path
    pub fn get_config_file_path(&self) -> &PathBuf {
        &self.config_file_path
    }

    /// Check if config file exists
    pub fn config_file_exists(&self) -> bool {
        self.config_file_path.exists()
    }

    /// Create a backup of the current config file
    pub fn backup_config(&self) -> Result<PathBuf, ConfigError> {
        if !self.config_file_path.exists() {
            return Err(ConfigError::ValidationError(
                "Cannot backup: config file does not exist".to_string()
            ));
        }

        let backup_path = self.config_file_path.with_extension("json.backup");
        fs::copy(&self.config_file_path, &backup_path)?;
        
        Ok(backup_path)
    }

    /// Restore config from backup
    pub fn restore_from_backup(&self) -> Result<AppConfig, ConfigError> {
        let backup_path = self.config_file_path.with_extension("json.backup");
        
        if !backup_path.exists() {
            return Err(ConfigError::ValidationError(
                "Backup file does not exist".to_string()
            ));
        }

        let content = fs::read_to_string(&backup_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        
        // Validate the backup config
        self.validate_config(&config)?;
        
        // Save the restored config as current
        self.save_config(&config)?;
        
        Ok(config)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::with_default_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_validation() {
        let config_manager = ConfigManager::with_default_path();
        let config = AppConfig::default();
        
        // Default config should be valid
        assert!(config_manager.validate_config(&config).is_ok());
    }

    #[test]
    fn test_invalid_itag() {
        let config_manager = ConfigManager::with_default_path();
        let mut config = AppConfig::default();
        config.itag = "invalid".to_string();
        
        assert!(config_manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_invalid_concurrent_limit() {
        let config_manager = ConfigManager::with_default_path();
        let mut config = AppConfig::default();
        config.concurrent_limit = 0;
        
        assert!(config_manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.json");
        let config_manager = ConfigManager::new(config_path);
        
        let original_config = AppConfig::default();
        
        // Save config
        assert!(config_manager.save_config(&original_config).is_ok());
        
        // Load config
        let loaded_config = config_manager.load_config().unwrap();
        
        // Configs should be equal
        assert_eq!(original_config.itag, loaded_config.itag);
        assert_eq!(original_config.concurrent_limit, loaded_config.concurrent_limit);
    }
}