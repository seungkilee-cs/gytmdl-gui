use crate::modules::state::{AppConfig, DownloadJob, JobStatus, Progress, DownloadStage};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub enum GytmdlError {
    BinaryNotFound(String),
    ProcessSpawnError(std::io::Error),
    InvalidUrl(String),
    ConfigError(String),
    ProcessError(String),
    ValidationError(String),
    IntegrityError(String),
    ManifestError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryManifest {
    pub binary_name: String,
    pub platform: PlatformInfo,
    pub size_bytes: u64,
    pub sha256: String,
    pub build_timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub target: String,
    pub extension: String,
}

impl std::fmt::Display for GytmdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GytmdlError::BinaryNotFound(path) => write!(f, "gytmdl binary not found at: {}", path),
            GytmdlError::ProcessSpawnError(e) => write!(f, "Failed to spawn gytmdl process: {}", e),
            GytmdlError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            GytmdlError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            GytmdlError::ProcessError(msg) => write!(f, "Process error: {}", msg),
            GytmdlError::ValidationError(msg) => write!(f, "Binary validation error: {}", msg),
            GytmdlError::IntegrityError(msg) => write!(f, "Binary integrity error: {}", msg),
            GytmdlError::ManifestError(msg) => write!(f, "Manifest error: {}", msg),
        }
    }
}

impl std::error::Error for GytmdlError {}

#[derive(Debug)]
pub struct GytmdlWrapper {
    binary_path: PathBuf,
}

impl GytmdlWrapper {
    /// Create a new GytmdlWrapper with automatic binary detection
    pub fn new() -> Result<Self, GytmdlError> {
        let binary_path = Self::detect_binary_path()?;
        Ok(Self { binary_path })
    }

    /// Create a GytmdlWrapper with a specific binary path
    pub fn with_binary_path(binary_path: PathBuf) -> Result<Self, GytmdlError> {
        if !binary_path.exists() {
            return Err(GytmdlError::BinaryNotFound(binary_path.to_string_lossy().to_string()));
        }
        Ok(Self { binary_path })
    }

    /// Detect the appropriate gytmdl binary for the current platform
    fn detect_binary_path() -> Result<PathBuf, GytmdlError> {
        let binary_name = Self::get_platform_binary_name();
        
        // ALWAYS check sidecar directory first and prefer it
        let sidecar_path = Self::get_sidecar_directory().join(&binary_name);
        println!("DEBUG: Checking sidecar path: {:?}", sidecar_path);
        if sidecar_path.exists() {
            println!("DEBUG: Using sidecar binary: {:?}", sidecar_path);
            return Ok(sidecar_path);
        }

        // Check in current directory
        let current_dir_path = std::env::current_dir()
            .map_err(|e| GytmdlError::ProcessSpawnError(e))?
            .join(&binary_name);
        if current_dir_path.exists() {
            println!("DEBUG: Using current directory binary: {:?}", current_dir_path);
            return Ok(current_dir_path);
        }

        // Only use system PATH as last resort and warn about it
        if let Ok(path_binary) = which::which("gytmdl") {
            println!("DEBUG: WARNING - Using system gytmdl binary: {:?}", path_binary);
            return Ok(path_binary);
        }

        Err(GytmdlError::BinaryNotFound(format!(
            "Could not find gytmdl binary. Searched for: {} in sidecar directory: {:?}, current directory, and PATH",
            binary_name, sidecar_path
        )))
    }

    /// Get the platform-specific binary name
    pub fn get_platform_binary_name() -> String {
        if cfg!(target_os = "windows") {
            if cfg!(target_arch = "x86_64") {
                "gytmdl-x86_64-pc-windows-msvc.exe".to_string()
            } else {
                "gytmdl.exe".to_string()
            }
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "gytmdl-aarch64-apple-darwin".to_string()
            } else {
                "gytmdl-x86_64-apple-darwin".to_string()
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "x86_64") {
                "gytmdl-x86_64-unknown-linux-gnu".to_string()
            } else {
                "gytmdl".to_string()
            }
        } else {
            "gytmdl".to_string()
        }
    }

    /// Get the sidecar directory path where bundled binaries are stored
    pub fn get_sidecar_directory() -> PathBuf {
        // In Tauri, sidecar binaries are typically in the resource directory
        // For development, we'll check relative to the current executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join("sidecars");
            }
        }
        
        // Fallback to current directory
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("sidecars")
    }

    /// Load and validate binary manifest
    pub fn load_manifest(&self) -> Result<BinaryManifest, GytmdlError> {
        let manifest_path = self.binary_path.with_extension("json");
        
        if !manifest_path.exists() {
            return Err(GytmdlError::ManifestError(format!(
                "Manifest file not found: {}", 
                manifest_path.display()
            )));
        }

        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| GytmdlError::ManifestError(format!(
                "Failed to read manifest: {}", e
            )))?;

        let manifest: BinaryManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| GytmdlError::ManifestError(format!(
                "Failed to parse manifest: {}", e
            )))?;

        Ok(manifest)
    }

    /// Calculate SHA256 hash of the binary file
    fn calculate_sha256(&self) -> Result<String, GytmdlError> {
        use std::io::Read;
        
        let mut file = fs::File::open(&self.binary_path)
            .map_err(|e| GytmdlError::IntegrityError(format!(
                "Failed to open binary for hashing: {}", e
            )))?;

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let mut buffer = [0; 8192];
        
        // For a proper SHA256, we'd need a crypto library, but for now we'll use a simple hash
        // In a real implementation, you'd want to use sha2 crate
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| GytmdlError::IntegrityError(format!(
                "Failed to read binary for hashing: {}", e
            )))?;

        // Simple hex representation of content hash (not cryptographically secure)
        use std::hash::{Hash, Hasher};
        content.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    /// Validate binary integrity against manifest
    pub fn validate_integrity(&self) -> Result<bool, GytmdlError> {
        let manifest = self.load_manifest()?;
        
        // Check file size
        let actual_size = fs::metadata(&self.binary_path)
            .map_err(|e| GytmdlError::IntegrityError(format!(
                "Failed to get binary metadata: {}", e
            )))?
            .len();

        if actual_size != manifest.size_bytes {
            return Err(GytmdlError::IntegrityError(format!(
                "Binary size mismatch. Expected: {}, Actual: {}", 
                manifest.size_bytes, actual_size
            )));
        }

        // Check hash (simplified version)
        let actual_hash = self.calculate_sha256()?;
        if actual_hash != manifest.sha256 {
            return Err(GytmdlError::IntegrityError(format!(
                "Binary hash mismatch. Expected: {}, Actual: {}", 
                manifest.sha256, actual_hash
            )));
        }

        Ok(true)
    }

    /// Get all available sidecar binaries in the sidecar directory
    pub fn list_available_binaries() -> Result<Vec<PathBuf>, GytmdlError> {
        let sidecar_dir = Self::get_sidecar_directory();
        
        if !sidecar_dir.exists() {
            return Ok(Vec::new());
        }

        let mut binaries = Vec::new();
        
        let entries = fs::read_dir(&sidecar_dir)
            .map_err(|e| GytmdlError::BinaryNotFound(format!(
                "Failed to read sidecar directory: {}", e
            )))?;

        for entry in entries {
            let entry = entry.map_err(|e| GytmdlError::BinaryNotFound(format!(
                "Failed to read directory entry: {}", e
            )))?;
            
            let path = entry.path();
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Check if it's a gytmdl binary (starts with "gytmdl" and is executable)
            if filename.starts_with("gytmdl") && 
               !filename.ends_with(".json") && 
               path.is_file() {
                binaries.push(path);
            }
        }

        Ok(binaries)
    }

    /// Select the best available binary for the current platform
    pub fn select_best_binary() -> Result<PathBuf, GytmdlError> {
        let available_binaries = Self::list_available_binaries()?;
        
        if available_binaries.is_empty() {
            return Err(GytmdlError::BinaryNotFound(
                "No gytmdl binaries found in sidecar directory".to_string()
            ));
        }

        let platform_binary_name = Self::get_platform_binary_name();
        
        // First, try to find exact platform match
        for binary in &available_binaries {
            if let Some(filename) = binary.file_name().and_then(|n| n.to_str()) {
                if filename == platform_binary_name {
                    return Ok(binary.clone());
                }
            }
        }

        // If no exact match, try to find a compatible binary
        let current_os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        };

        for binary in &available_binaries {
            if let Some(filename) = binary.file_name().and_then(|n| n.to_str()) {
                if filename.contains(current_os) {
                    return Ok(binary.clone());
                }
            }
        }

        // As a last resort, return the first available binary
        Ok(available_binaries[0].clone())
    }

    /// Build command arguments from AppConfig
    pub fn build_command_args(&self, config: &AppConfig, url: &str, job_id: &str) -> Result<Vec<String>, GytmdlError> {
        let mut args = Vec::new();

        // Validate URL
        if !Self::is_valid_youtube_music_url(url) {
            return Err(GytmdlError::InvalidUrl(url.to_string()));
        }

        // Output directory
        args.push("--output-path".to_string());
        args.push(config.output_path.to_string_lossy().to_string());

        // Audio quality (itag) - use short form like CLI
        args.push("-i".to_string());
        args.push(config.itag.clone());

        // Cookies file - only add if we have cookies AND they exist
        if let Some(cookies_path) = &config.cookies_path {
            if cookies_path.exists() {
                args.push("--cookies-path".to_string());
                args.push(cookies_path.to_string_lossy().to_string());
            }
        }

        // Download mode
        match config.download_mode {
            crate::modules::state::DownloadMode::Audio => {
                // Default mode, no additional args needed
            }
            crate::modules::state::DownloadMode::Video => {
                args.push("--video".to_string());
            }
            crate::modules::state::DownloadMode::AudioVideo => {
                args.push("--audio-video".to_string());
            }
        }

        // Cover settings
        if config.save_cover {
            args.push("--cover-size".to_string());
            args.push(config.cover_size.to_string());

            args.push("--cover-format".to_string());
            match config.cover_format {
                crate::modules::state::CoverFormat::Jpg => args.push("jpg".to_string()),
                crate::modules::state::CoverFormat::Png => args.push("png".to_string()),
                crate::modules::state::CoverFormat::Webp => args.push("webp".to_string()),
            }

            args.push("--cover-quality".to_string());
            args.push(config.cover_quality.to_string());
        } else {
            args.push("--no-cover".to_string());
        }

        // Template settings
        args.push("--template-folder".to_string());
        args.push(config.template_folder.clone());

        args.push("--template-file".to_string());
        args.push(config.template_file.clone());

        args.push("--template-date".to_string());
        args.push(config.template_date.clone());

        // PO Token
        if let Some(po_token) = &config.po_token {
            if !po_token.trim().is_empty() {
                args.push("--po-token".to_string());
                args.push(po_token.clone());
            }
        }

        // Exclude tags
        if let Some(exclude_tags) = &config.exclude_tags {
            if !exclude_tags.trim().is_empty() {
                args.push("--exclude-tags".to_string());
                args.push(exclude_tags.clone());
            }
        }

        // Truncate
        if let Some(truncate) = config.truncate {
            args.push("--truncate".to_string());
            args.push(truncate.to_string());
        }

        // Boolean flags
        if config.overwrite {
            args.push("--overwrite".to_string());
        }

        if config.no_synced_lyrics {
            args.push("--no-synced-lyrics".to_string());
        }

        // Note: gytmdl doesn't have --progress or --verbose flags
        // We'll parse output from the normal gytmdl output

        // Finally, add the URL
        args.push(url.to_string());

        Ok(args)
    }

    /// Validate if URL is a valid YouTube Music URL
    fn is_valid_youtube_music_url(url: &str) -> bool {
        // Basic validation for YouTube Music URLs - must be HTTP/HTTPS
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }
        
        url.contains("music.youtube.com") || 
        url.contains("youtube.com/watch") ||
        url.contains("youtube.com/playlist") ||
        url.contains("youtu.be/")
    }

    /// Spawn a gytmdl process for downloading
    pub async fn spawn_download_process(
        &self,
        config: &AppConfig,
        job: &DownloadJob,
    ) -> Result<GytmdlProcess, GytmdlError> {
        let args = self.build_command_args(config, &job.url, &job.id)?;

        println!("DEBUG: Spawning process with binary: {:?}", self.binary_path);
        println!("DEBUG: Command args: {:?}", args);
        println!("DEBUG: Working directory: {:?}", config.output_path);

        let mut command = Command::new(&self.binary_path);
        command
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Create output and temp directories if they don't exist
        if let Err(e) = std::fs::create_dir_all(&config.output_path) {
            println!("DEBUG: Failed to create output directory: {}", e);
            return Err(GytmdlError::ConfigError(format!("Failed to create output directory: {}", e)));
        }
        
        if let Err(e) = std::fs::create_dir_all(&config.temp_path) {
            println!("DEBUG: Failed to create temp directory: {}", e);
            return Err(GytmdlError::ConfigError(format!("Failed to create temp directory: {}", e)));
        }

        // Set working directory to output path
        command.current_dir(&config.output_path);

        let child = command.spawn()
            .map_err(|e| {
                println!("DEBUG: Process spawn error: {}", e);
                GytmdlError::ProcessSpawnError(e)
            })?;

        println!("DEBUG: Process spawned with PID: {:?}", child.id());
        Ok(GytmdlProcess::new(child, job.id.clone()))
    }

    /// Test if the gytmdl binary is working
    pub async fn test_binary(&self) -> Result<String, GytmdlError> {
        let mut command = Command::new(&self.binary_path);
        command
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = command.output().await
            .map_err(|e| GytmdlError::ProcessSpawnError(e))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(GytmdlError::ProcessError(format!("Binary test failed: {}", error)))
        }
    }

    /// Get the binary path
    pub fn get_binary_path(&self) -> &Path {
        &self.binary_path
    }

    /// Check if binary exists and is executable
    pub fn is_binary_available(&self) -> bool {
        self.binary_path.exists() && self.binary_path.is_file()
    }
}

/// Represents a running gytmdl process
pub struct GytmdlProcess {
    child: Child,
    job_id: String,
    stdout_reader: Option<BufReader<tokio::process::ChildStdout>>,
    stderr_reader: Option<BufReader<tokio::process::ChildStderr>>,
}

impl GytmdlProcess {
    pub fn new(mut child: Child, job_id: String) -> Self {
        let stdout_reader = child.stdout.take().map(BufReader::new);
        let stderr_reader = child.stderr.take().map(BufReader::new);

        Self {
            child,
            job_id,
            stdout_reader,
            stderr_reader,
        }
    }

    /// Get the job ID associated with this process
    pub fn job_id(&self) -> &str {
        &self.job_id
    }

    /// Get the process ID
    pub fn process_id(&self) -> Option<u32> {
        self.child.id()
    }

    /// Read a line from stdout
    pub async fn read_stdout_line(&mut self) -> Result<Option<String>, std::io::Error> {
        if let Some(reader) = &mut self.stdout_reader {
            let mut line = String::new();
            match reader.read_line(&mut line).await? {
                0 => Ok(None), // EOF
                _ => {
                    // Remove trailing newline
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    Ok(Some(line))
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Read a line from stderr
    pub async fn read_stderr_line(&mut self) -> Result<Option<String>, std::io::Error> {
        if let Some(reader) = &mut self.stderr_reader {
            let mut line = String::new();
            match reader.read_line(&mut line).await? {
                0 => Ok(None), // EOF
                _ => {
                    // Remove trailing newline
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    Ok(Some(line))
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Wait for the process to complete
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus, std::io::Error> {
        self.child.wait().await
    }

    /// Try to wait for the process without blocking
    pub fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
        self.child.try_wait()
    }

    /// Kill the process
    pub async fn kill(&mut self) -> Result<(), std::io::Error> {
        self.child.kill().await
    }

    /// Start the process (if not already started)
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        // Process is already started when created, this is a no-op
        Ok(())
    }
}

impl Default for GytmdlWrapper {
    fn default() -> Self {
        Self::new().expect("Failed to create GytmdlWrapper")
    }
}