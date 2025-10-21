use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use serde::{Deserialize, Serialize};
// chrono is used in analyze_cookies method for timestamp comparison

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieInfo {
    pub is_valid: bool,
    pub expiration_warning: Option<String>,
    pub po_token_present: bool,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum CookieError {
    FileNotFound(PathBuf),
    InvalidFormat(String),
    ReadError(io::Error),
    ValidationError(String),
}

impl std::fmt::Display for CookieError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookieError::FileNotFound(path) => write!(f, "Cookie file not found: {}", path.display()),
            CookieError::InvalidFormat(msg) => write!(f, "Invalid cookie format: {}", msg),
            CookieError::ReadError(e) => write!(f, "Failed to read cookie file: {}", e),
            CookieError::ValidationError(msg) => write!(f, "Cookie validation failed: {}", msg),
        }
    }
}

impl std::error::Error for CookieError {}

pub struct CookieManager {
    cookies_dir: PathBuf,
}

impl CookieManager {
    pub fn new() -> Self {
        let cookies_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".gytmdl-gui")
            .join("cookies");
        
        Self { cookies_dir }
    }

    pub fn with_cookies_dir(cookies_dir: PathBuf) -> Self {
        Self { cookies_dir }
    }

    /// Import cookies from a file
    pub async fn import_cookies(&self, source_path: &Path) -> Result<CookieInfo, CookieError> {
        // Check if source file exists
        if !source_path.exists() {
            return Err(CookieError::FileNotFound(source_path.to_path_buf()));
        }

        // Read and validate the cookie file
        let content = fs::read_to_string(source_path)
            .map_err(CookieError::ReadError)?;

        // Validate cookie format
        self.validate_cookie_content(&content)?;

        // Create cookies directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&self.cookies_dir) {
            return Err(CookieError::ReadError(e));
        }

        // Copy cookies to our managed location
        let target_path = self.cookies_dir.join("cookies.txt");
        if let Err(e) = fs::copy(source_path, &target_path) {
            return Err(CookieError::ReadError(e));
        }

        // Analyze the cookies
        let cookie_info = self.analyze_cookies(&content)?;

        Ok(CookieInfo {
            is_valid: cookie_info.is_valid,
            expiration_warning: cookie_info.expiration_warning,
            po_token_present: cookie_info.po_token_present,
            file_path: Some(target_path),
        })
    }

    /// Validate cookies from the managed location
    pub async fn validate_cookies(&self) -> Result<CookieInfo, CookieError> {
        let cookie_path = self.cookies_dir.join("cookies.txt");
        
        if !cookie_path.exists() {
            return Ok(CookieInfo {
                is_valid: false,
                expiration_warning: Some("No cookies file found".to_string()),
                po_token_present: false,
                file_path: None,
            });
        }

        let content = fs::read_to_string(&cookie_path)
            .map_err(CookieError::ReadError)?;

        let cookie_info = self.analyze_cookies(&content)?;

        Ok(CookieInfo {
            is_valid: cookie_info.is_valid,
            expiration_warning: cookie_info.expiration_warning,
            po_token_present: cookie_info.po_token_present,
            file_path: Some(cookie_path),
        })
    }

    /// Get the path to the managed cookies file
    pub fn get_cookies_path(&self) -> PathBuf {
        self.cookies_dir.join("cookies.txt")
    }

    /// Remove the managed cookies file
    pub async fn clear_cookies(&self) -> Result<(), CookieError> {
        let cookie_path = self.get_cookies_path();
        if cookie_path.exists() {
            fs::remove_file(cookie_path)
                .map_err(CookieError::ReadError)?;
        }
        Ok(())
    }

    /// Validate cookie file content format
    fn validate_cookie_content(&self, content: &str) -> Result<(), CookieError> {
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return Err(CookieError::InvalidFormat("Cookie file is empty".to_string()));
        }

        // Check for Netscape format header (optional but common)
        // Skip comment lines and empty lines
        let mut valid_cookie_lines = 0;
        let mut has_youtube_cookies = false;

        for line in lines.iter() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Validate cookie line format (tab-separated values)
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 6 {
                return Err(CookieError::InvalidFormat(
                    format!("Invalid cookie line format: expected at least 6 tab-separated fields, got {}", parts.len())
                ));
            }

            // Check if this is a YouTube-related cookie
            if parts[0].contains("youtube.com") || parts[0].contains(".youtube.com") {
                has_youtube_cookies = true;
            }

            valid_cookie_lines += 1;
        }

        if valid_cookie_lines == 0 {
            return Err(CookieError::InvalidFormat("No valid cookie lines found".to_string()));
        }

        if !has_youtube_cookies {
            return Err(CookieError::ValidationError(
                "No YouTube cookies found. Make sure to export cookies from youtube.com or music.youtube.com".to_string()
            ));
        }

        Ok(())
    }

    /// Analyze cookies and provide information about their status
    fn analyze_cookies(&self, content: &str) -> Result<CookieInfo, CookieError> {
        let mut has_youtube_cookies = false;
        let mut has_po_token = false;
        let mut expiration_warnings = Vec::new();
        let current_time = chrono::Utc::now().timestamp();

        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 7 {
                continue; // Skip malformed lines
            }

            let domain = parts[0];
            let name = parts[5];
            let value = parts[6];

            // Check for YouTube domain
            if domain.contains("youtube.com") {
                has_youtube_cookies = true;

                // Check for PO token
                if name == "__Secure-YT-Core-PO-Token" || name.contains("PO") {
                    has_po_token = true;
                }

                // Check expiration (parts[4] is expiration timestamp)
                if let Ok(expiration) = parts[4].parse::<i64>() {
                    let days_until_expiration = (expiration - current_time) / 86400; // seconds to days
                    
                    if days_until_expiration < 0 {
                        expiration_warnings.push(format!("Cookie '{}' has expired", name));
                    } else if days_until_expiration < 7 {
                        expiration_warnings.push(format!("Cookie '{}' expires in {} days", name, days_until_expiration));
                    }
                }

                // Check for important YouTube cookies
                if name == "SAPISID" || name == "HSID" || name == "SSID" {
                    if value.is_empty() {
                        expiration_warnings.push(format!("Important cookie '{}' is empty", name));
                    }
                }
            }
        }

        let expiration_warning = if expiration_warnings.is_empty() {
            None
        } else {
            Some(expiration_warnings.join("; "))
        };

        Ok(CookieInfo {
            is_valid: has_youtube_cookies,
            expiration_warning,
            po_token_present: has_po_token,
            file_path: None, // Will be set by caller
        })
    }
}

impl Default for CookieManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_cookie_manager_creation() {
        let manager = CookieManager::new();
        let path = manager.get_cookies_path();
        assert!(path.to_string_lossy().contains("cookies.txt"));
    }

    #[tokio::test]
    async fn test_validate_cookie_content_valid() {
        let manager = CookieManager::new();
        let valid_content = "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t1234567890\tSAPISID\ttest_value\n.youtube.com\tTRUE\t/\tTRUE\t1234567890\tHSID\ttest_value2";
        
        let result = manager.validate_cookie_content(valid_content);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_cookie_content_invalid_empty() {
        let manager = CookieManager::new();
        let result = manager.validate_cookie_content("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_validate_cookie_content_invalid_format() {
        let manager = CookieManager::new();
        let invalid_content = "invalid\tformat";
        
        let result = manager.validate_cookie_content(invalid_content);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_cookie_content_no_youtube() {
        let manager = CookieManager::new();
        let content = ".example.com\tTRUE\t/\tTRUE\t1234567890\ttest\tvalue";
        
        let result = manager.validate_cookie_content(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No YouTube cookies"));
    }

    #[tokio::test]
    async fn test_import_cookies_file_not_found() {
        let temp_dir = tempdir().unwrap();
        let manager = CookieManager::with_cookies_dir(temp_dir.path().to_path_buf());
        let non_existent = temp_dir.path().join("non_existent.txt");
        
        let result = manager.import_cookies(&non_existent).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CookieError::FileNotFound(_)));
    }

    #[tokio::test]
    async fn test_import_cookies_success() {
        let temp_dir = tempdir().unwrap();
        let manager = CookieManager::with_cookies_dir(temp_dir.path().join("cookies"));
        
        // Create a valid cookie file
        let source_file = temp_dir.path().join("source_cookies.txt");
        let valid_content = "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t9999999999\tSAPISID\ttest_value\n.youtube.com\tTRUE\t/\tTRUE\t9999999999\t__Secure-YT-Core-PO-Token\tpo_token_value";
        fs::write(&source_file, valid_content).unwrap();
        
        let result = manager.import_cookies(&source_file).await;
        assert!(result.is_ok());
        
        let cookie_info = result.unwrap();
        assert!(cookie_info.is_valid);
        assert!(cookie_info.po_token_present);
        assert!(cookie_info.file_path.is_some());
    }

    #[tokio::test]
    async fn test_validate_cookies_no_file() {
        let temp_dir = tempdir().unwrap();
        let manager = CookieManager::with_cookies_dir(temp_dir.path().to_path_buf());
        
        let result = manager.validate_cookies().await;
        assert!(result.is_ok());
        
        let cookie_info = result.unwrap();
        assert!(!cookie_info.is_valid);
        assert!(cookie_info.expiration_warning.is_some());
        assert!(!cookie_info.po_token_present);
        assert!(cookie_info.file_path.is_none());
    }

    #[tokio::test]
    async fn test_clear_cookies() {
        let temp_dir = tempdir().unwrap();
        let manager = CookieManager::with_cookies_dir(temp_dir.path().to_path_buf());
        
        // Create cookies directory and file
        fs::create_dir_all(temp_dir.path()).unwrap();
        let cookie_file = manager.get_cookies_path();
        fs::write(&cookie_file, "test content").unwrap();
        assert!(cookie_file.exists());
        
        // Clear cookies
        let result = manager.clear_cookies().await;
        assert!(result.is_ok());
        assert!(!cookie_file.exists());
    }

    #[tokio::test]
    async fn test_analyze_cookies_with_po_token() {
        let manager = CookieManager::new();
        let content = ".youtube.com\tTRUE\t/\tTRUE\t9999999999\t__Secure-YT-Core-PO-Token\tpo_value\n.youtube.com\tTRUE\t/\tTRUE\t9999999999\tSAPISID\ttest_value";
        
        let result = manager.analyze_cookies(content);
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.is_valid);
        assert!(info.po_token_present);
    }

    #[tokio::test]
    async fn test_analyze_cookies_expiration_warning() {
        let manager = CookieManager::new();
        // Use a timestamp that's already expired (timestamp 1)
        let content = ".youtube.com\tTRUE\t/\tTRUE\t1\tSAPISID\ttest_value";
        
        let result = manager.analyze_cookies(content);
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert!(info.is_valid);
        assert!(info.expiration_warning.is_some());
        assert!(info.expiration_warning.unwrap().contains("expired"));
    }
}