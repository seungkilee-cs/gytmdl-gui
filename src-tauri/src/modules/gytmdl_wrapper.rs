use crate::modules::state::{AppConfig, DownloadJob, JobStatus, Progress, DownloadStage};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum GytmdlError {
    BinaryNotFound(String),
    ProcessSpawnError(std::io::Error),
    InvalidUrl(String),
    ConfigError(String),
    ProcessError(String),
}

impl std::fmt::Display for GytmdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GytmdlError::BinaryNotFound(path) => write!(f, "gytmdl binary not found at: {}", path),
            GytmdlError::ProcessSpawnError(e) => write!(f, "Failed to spawn gytmdl process: {}", e),
            GytmdlError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            GytmdlError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            GytmdlError::ProcessError(msg) => write!(f, "Process error: {}", msg),
        }
    }
}

impl std::error::Error for GytmdlError {}

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
        
        // Check in the sidecar directory first (bundled with app)
        let sidecar_path = Self::get_sidecar_directory().join(&binary_name);
        if sidecar_path.exists() {
            return Ok(sidecar_path);
        }

        // Check in current directory
        let current_dir_path = std::env::current_dir()
            .map_err(|e| GytmdlError::ProcessSpawnError(e))?
            .join(&binary_name);
        if current_dir_path.exists() {
            return Ok(current_dir_path);
        }

        // Check in PATH
        if let Ok(path_binary) = which::which(&binary_name) {
            return Ok(path_binary);
        }

        // Check for generic "gytmdl" in PATH as fallback
        if let Ok(path_binary) = which::which("gytmdl") {
            return Ok(path_binary);
        }

        Err(GytmdlError::BinaryNotFound(format!(
            "Could not find gytmdl binary. Searched for: {} in sidecar directory, current directory, and PATH",
            binary_name
        )))
    }

    /// Get the platform-specific binary name
    fn get_platform_binary_name() -> String {
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
    fn get_sidecar_directory() -> PathBuf {
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

        // Temp directory
        args.push("--temp-path".to_string());
        args.push(config.temp_path.to_string_lossy().to_string());

        // Cookies file
        if let Some(cookies_path) = &config.cookies_path {
            args.push("--cookies-path".to_string());
            args.push(cookies_path.to_string_lossy().to_string());
        }

        // Audio quality (itag)
        args.push("--itag".to_string());
        args.push(config.itag.clone());

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

        // Add progress output for parsing
        args.push("--progress".to_string());
        args.push("--verbose".to_string());

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

        let mut command = Command::new(&self.binary_path);
        command
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Set working directory to output path
        command.current_dir(&config.output_path);

        let child = command.spawn()
            .map_err(|e| GytmdlError::ProcessSpawnError(e))?;

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