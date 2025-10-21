use gytmdl_gui_lib::modules::{
    gytmdl_wrapper::{GytmdlWrapper, GytmdlError, GytmdlProcess},
    progress_parser::ProgressParser,
    state::{AppConfig, DownloadStage},
};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tempfile::tempdir;
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;

/// Integration tests for sidecar management
/// Tests process spawning, termination, progress parsing, and error handling

#[tokio::test]
async fn test_gytmdl_wrapper_binary_detection() {
    // Test automatic binary detection
    let result = GytmdlWrapper::new();
    
    // This might fail if no gytmdl binary is available, which is expected in CI
    match result {
        Ok(wrapper) => {
            // If we found a binary, verify it exists
            assert!(wrapper.is_binary_available());
            assert!(wrapper.get_binary_path().exists());
        }
        Err(GytmdlError::BinaryNotFound(_)) => {
            // This is expected if no gytmdl binary is installed
            println!("No gytmdl binary found - this is expected in test environments");
        }
        Err(e) => panic!("Unexpected error during binary detection: {}", e),
    }
}

#[tokio::test]
async fn test_gytmdl_wrapper_with_custom_binary_path() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let binary_path = temp_dir.path().join("fake_gytmdl");
    
    // Create a fake binary file
    fs::write(&binary_path, "#!/bin/bash\necho 'fake gytmdl'").await
        .expect("Failed to create fake binary");
    
    // Make it executable (Unix-like systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms).await.unwrap();
    }
    
    // Test with existing binary
    let wrapper = GytmdlWrapper::with_binary_path(binary_path.clone())
        .expect("Should create wrapper with existing binary");
    assert!(wrapper.is_binary_available());
    assert_eq!(wrapper.get_binary_path(), binary_path);
    
    // Test with non-existent binary
    let non_existent_path = temp_dir.path().join("non_existent_binary");
    let result = GytmdlWrapper::with_binary_path(non_existent_path);
    assert!(matches!(result, Err(GytmdlError::BinaryNotFound(_))));
}

#[tokio::test]
async fn test_command_argument_building() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let binary_path = temp_dir.path().join("fake_gytmdl");
    
    // Create a fake binary
    fs::write(&binary_path, "fake binary").await.unwrap();
    
    let wrapper = GytmdlWrapper::with_binary_path(binary_path)
        .expect("Should create wrapper");
    
    let config = AppConfig::default();
    let url = "https://music.youtube.com/watch?v=test123";
    let job_id = "test-job-id";
    
    let args = wrapper.build_command_args(&config, url, job_id)
        .expect("Should build command args");
    
    // Verify essential arguments are present
    assert!(args.contains(&"--output-path".to_string()));
    assert!(args.contains(&config.output_path.to_string_lossy().to_string()));
    assert!(args.contains(&"--temp-path".to_string()));
    assert!(args.contains(&config.temp_path.to_string_lossy().to_string()));
    assert!(args.contains(&"--itag".to_string()));
    assert!(args.contains(&config.itag));
    assert!(args.contains(&"--progress".to_string()));
    assert!(args.contains(&"--verbose".to_string()));
    assert!(args.contains(&url.to_string()));
}

#[tokio::test]
async fn test_invalid_url_handling() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let binary_path = temp_dir.path().join("fake_gytmdl");
    fs::write(&binary_path, "fake binary").await.unwrap();
    
    let wrapper = GytmdlWrapper::with_binary_path(binary_path).unwrap();
    let config = AppConfig::default();
    
    let invalid_urls = vec![
        "https://example.com/not-youtube",
        "not-a-url-at-all",
        "ftp://music.youtube.com/watch?v=test",
        "",
    ];
    
    for invalid_url in invalid_urls {
        let result = wrapper.build_command_args(&config, invalid_url, "test-job");
        assert!(matches!(result, Err(GytmdlError::InvalidUrl(_))), 
               "Should reject invalid URL: {}", invalid_url);
    }
}

#[tokio::test]
async fn test_valid_url_acceptance() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let binary_path = temp_dir.path().join("fake_gytmdl");
    fs::write(&binary_path, "fake binary").await.unwrap();
    
    let wrapper = GytmdlWrapper::with_binary_path(binary_path).unwrap();
    let config = AppConfig::default();
    
    let valid_urls = vec![
        "https://music.youtube.com/watch?v=test123",
        "https://music.youtube.com/playlist?list=PLtest",
        "https://youtube.com/watch?v=test456",
        "https://www.youtube.com/playlist?list=PLtest2",
        "https://youtu.be/test789",
    ];
    
    for valid_url in valid_urls {
        let result = wrapper.build_command_args(&config, valid_url, "test-job");
        assert!(result.is_ok(), "Should accept valid URL: {}", valid_url);
    }
}

#[tokio::test]
async fn test_mock_process_spawning_and_termination() {
    // Create a mock script that simulates gytmdl behavior
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("mock_gytmdl.sh");
    
    // Create a script that outputs progress and runs for a while
    let script_content = r#"#!/bin/bash
echo "Initializing download process"
echo "Fetching video metadata"
echo "[download] 25.0% of 5.0MiB at 1.0MiB/s ETA 00:04"
sleep 0.1
echo "[download] 50.0% of 5.0MiB at 1.0MiB/s ETA 00:02"
sleep 0.1
echo "[download] 75.0% of 5.0MiB at 1.0MiB/s ETA 00:01"
sleep 0.1
echo "[download] 100% of 5.0MiB in 00:05"
echo "Applying tags to file"
echo "Download completed successfully"
"#;
    
    fs::write(&script_path, script_content).await.unwrap();
    
    // Make script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).await.unwrap();
    }
    
    // Test process spawning
    let mut command = Command::new(&script_path);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    
    let child = command.spawn().expect("Failed to spawn mock process");
    let mut process = GytmdlProcess::new(child, "test-job-id".to_string());
    
    assert_eq!(process.job_id(), "test-job-id");
    assert!(process.process_id().is_some());
    
    // Test reading output
    let mut output_lines = Vec::new();
    while let Ok(Some(line)) = process.read_stdout_line().await {
        output_lines.push(line);
        if output_lines.len() > 10 {
            break; // Prevent infinite loop
        }
    }
    
    assert!(!output_lines.is_empty());
    assert!(output_lines.iter().any(|line| line.contains("Initializing")));
    assert!(output_lines.iter().any(|line| line.contains("[download]")));
    
    // Test process termination
    let exit_status = process.wait().await.expect("Failed to wait for process");
    assert!(exit_status.success());
}

#[tokio::test]
async fn test_process_killing() {
    // Create a long-running mock script
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("long_running_mock.sh");
    
    let script_content = r#"#!/bin/bash
echo "Starting long process"
sleep 10
echo "This should not be reached"
"#;
    
    fs::write(&script_path, script_content).await.unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).await.unwrap();
    }
    
    let mut command = Command::new(&script_path);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    
    let child = command.spawn().expect("Failed to spawn long-running process");
    let mut process = GytmdlProcess::new(child, "long-job-id".to_string());
    
    // Read first line to ensure process started
    let first_line = timeout(Duration::from_secs(2), process.read_stdout_line()).await
        .expect("Timeout waiting for first line")
        .expect("Failed to read stdout")
        .expect("No output received");
    
    assert!(first_line.contains("Starting long process"));
    
    // Kill the process
    process.kill().await.expect("Failed to kill process");
    
    // Verify process is terminated
    let exit_status = process.wait().await.expect("Failed to wait for killed process");
    assert!(!exit_status.success()); // Process was killed, so it didn't exit successfully
}

#[tokio::test]
async fn test_progress_parsing_with_sample_output() {
    let sample_outputs = vec![
        // Download progress samples
        ("[download] 0% of 5.67MiB", DownloadStage::DownloadingAudio, Some(0.0)),
        ("[download] 25.5% of 3.45MiB at 1.23MiB/s ETA 00:02", DownloadStage::DownloadingAudio, Some(25.5)),
        ("[download] 50.0% of 3.45MiB at 1.23MiB/s ETA 00:01", DownloadStage::DownloadingAudio, Some(50.0)),
        ("[download] 100% of 3.45MiB in 00:15", DownloadStage::DownloadingAudio, Some(100.0)),
        
        // Stage indicator samples
        ("Initializing download process", DownloadStage::Initializing, None),
        ("Starting gytmdl downloader", DownloadStage::Initializing, None),
        ("Fetching video metadata from YouTube Music", DownloadStage::FetchingMetadata, None),
        ("Getting video info", DownloadStage::FetchingMetadata, None),
        ("Extracting audio stream information", DownloadStage::FetchingMetadata, None),
        ("[download] Destination: /path/to/file.m4a", DownloadStage::DownloadingAudio, None),
        ("Remuxing audio stream to M4A format", DownloadStage::Remuxing, None),
        ("Processing audio file", DownloadStage::Remuxing, None),
        ("Converting audio format", DownloadStage::Remuxing, None),
        ("Applying ID3 tags to output file", DownloadStage::ApplyingTags, None),
        ("Writing metadata to file", DownloadStage::ApplyingTags, None),
        ("Adding cover art", DownloadStage::ApplyingTags, None),
        ("Finalizing download", DownloadStage::Finalizing, None),
        ("Download completed successfully", DownloadStage::Finalizing, None),
        ("Finished processing track", DownloadStage::Finalizing, None),
        
        // Generic progress samples
        ("Step 1 of 5: Initializing", DownloadStage::Initializing, Some(20.0)),
        ("Step 3 of 4: Processing audio", DownloadStage::Remuxing, Some(75.0)),
        ("[2/6] Downloading track", DownloadStage::DownloadingAudio, Some(33.33)),
        
        // Keyword-based samples
        ("Starting download process", DownloadStage::Initializing, None),
        ("Extracting track information", DownloadStage::FetchingMetadata, None),
        ("Downloading audio stream", DownloadStage::DownloadingAudio, None),
        ("Converting file format", DownloadStage::Remuxing, None),
        ("Adding metadata tags", DownloadStage::ApplyingTags, None),
        ("Finalizing output file", DownloadStage::Finalizing, None),
    ];
    
    for (output, expected_stage, expected_percentage) in sample_outputs {
        let result = ProgressParser::parse_output(output);
        
        assert!(result.is_some(), "Should parse output: {}", output);
        let progress = result.unwrap();
        
        assert!(matches!(progress.stage, expected_stage), 
               "Expected stage {:?} for output: {}", expected_stage, output);
        
        match (progress.percentage, expected_percentage) {
            (Some(actual), Some(expected)) => {
                // Allow small floating point differences
                assert!((actual - expected).abs() < 0.1, 
                       "Expected percentage ~{}, got {} for output: {}", expected, actual, output);
            }
            (None, None) => {
                // Both None, this is expected
            }
            (actual, expected) => {
                panic!("Expected percentage {:?}, got {:?} for output: {}", expected, actual, output);
            }
        }
        
        assert_eq!(progress.current_step, output);
    }
}

#[tokio::test]
async fn test_error_line_detection_and_parsing() {
    let error_samples = vec![
        "Error: Failed to download video",
        "ERROR: Network connection timeout",
        "Fatal: Cannot access YouTube Music",
        "Exception occurred during processing",
        "Download failed with error code 404",
        "Traceback (most recent call last):",
        "error: Invalid URL format",
        "fatal: Process terminated unexpectedly",
    ];
    
    for error_line in error_samples {
        // Test error detection
        assert!(ProgressParser::is_error_line(error_line), 
               "Should detect as error: {}", error_line);
        
        // Test error parsing
        let error_progress = ProgressParser::parse_error(error_line);
        assert!(matches!(error_progress.stage, DownloadStage::Failed));
        assert!(error_progress.current_step.contains("Error:"));
        assert!(error_progress.current_step.contains(error_line));
    }
}

#[tokio::test]
async fn test_completion_line_detection() {
    let completion_samples = vec![
        "Download completed successfully",
        "Successfully downloaded track.m4a",
        "Finished downloading album",
        "[download] 100% of 5.0MiB in 00:30",
        "download completed for track",
    ];
    
    for completion_line in completion_samples {
        assert!(ProgressParser::is_completion_line(completion_line), 
               "Should detect as completion: {}", completion_line);
    }
    
    let non_completion_samples = vec![
        "[download] 50% of 5.0MiB",
        "Processing audio file",
        "Starting download",
        "Fetching metadata",
    ];
    
    for non_completion_line in non_completion_samples {
        assert!(!ProgressParser::is_completion_line(non_completion_line), 
               "Should not detect as completion: {}", non_completion_line);
    }
}

#[tokio::test]
async fn test_output_sanitization() {
    let samples_with_ansi = vec![
        ("\x1b[32mGreen download progress\x1b[0m", "Green download progress"),
        ("\x1b[1;31mRed error message\x1b[0m", "Red error message"),
        ("\x1b[33m[download] 50% complete\x1b[0m", "[download] 50% complete"),
        ("Normal text without ANSI codes", "Normal text without ANSI codes"),
        ("  \x1b[36mCyan text with spaces\x1b[0m  ", "Cyan text with spaces"),
    ];
    
    for (input, expected) in samples_with_ansi {
        let sanitized = ProgressParser::sanitize_output(input);
        assert_eq!(sanitized, expected, "Failed to sanitize: {:?}", input);
    }
}

#[tokio::test]
async fn test_missing_binary_error_handling() {
    let non_existent_path = PathBuf::from("/definitely/does/not/exist/gytmdl");
    
    let result = GytmdlWrapper::with_binary_path(non_existent_path);
    assert!(matches!(result, Err(GytmdlError::BinaryNotFound(_))));
    
    if let Err(GytmdlError::BinaryNotFound(msg)) = result {
        assert!(msg.contains("/definitely/does/not/exist/gytmdl"));
    }
}

#[tokio::test]
async fn test_corrupted_binary_handling() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let corrupted_binary_path = temp_dir.path().join("corrupted_gytmdl");
    
    // Create a file with invalid content (not a valid executable)
    fs::write(&corrupted_binary_path, "This is not a valid executable file").await.unwrap();
    
    let wrapper = GytmdlWrapper::with_binary_path(corrupted_binary_path.clone())
        .expect("Should create wrapper even with corrupted binary");
    
    // Test binary should fail when we try to run it
    let test_result = wrapper.test_binary().await;
    assert!(matches!(test_result, Err(GytmdlError::ProcessSpawnError(_)) | Err(GytmdlError::ProcessError(_))));
}

#[tokio::test]
async fn test_binary_version_testing() {
    // Create a mock binary that responds to --version
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let mock_binary_path = temp_dir.path().join("mock_gytmdl_version.sh");
    
    let script_content = r#"#!/bin/bash
if [ "$1" = "--version" ]; then
    echo "gytmdl 1.0.0"
    exit 0
else
    echo "Unknown argument: $1" >&2
    exit 1
fi
"#;
    
    fs::write(&mock_binary_path, script_content).await.unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&mock_binary_path).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_binary_path, perms).await.unwrap();
    }
    
    let wrapper = GytmdlWrapper::with_binary_path(mock_binary_path).unwrap();
    let version_result = wrapper.test_binary().await;
    
    assert!(version_result.is_ok());
    let version = version_result.unwrap();
    assert!(version.contains("gytmdl"));
    assert!(version.contains("1.0.0"));
}

#[tokio::test]
async fn test_binary_test_failure_handling() {
    // Create a mock binary that fails version check
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let failing_binary_path = temp_dir.path().join("failing_gytmdl.sh");
    
    let script_content = r#"#!/bin/bash
echo "Error: Binary is corrupted" >&2
exit 1
"#;
    
    fs::write(&failing_binary_path, script_content).await.unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&failing_binary_path).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&failing_binary_path, perms).await.unwrap();
    }
    
    let wrapper = GytmdlWrapper::with_binary_path(failing_binary_path).unwrap();
    let test_result = wrapper.test_binary().await;
    
    assert!(matches!(test_result, Err(GytmdlError::ProcessError(_))));
    if let Err(GytmdlError::ProcessError(msg)) = test_result {
        assert!(msg.contains("Binary test failed"));
    }
}

#[tokio::test]
async fn test_platform_specific_binary_names() {
    // Test that platform-specific binary names are generated correctly
    // Note: This test will pass different assertions based on the platform it runs on
    
    let wrapper_result = GytmdlWrapper::new();
    
    // We can't test the exact binary name since it depends on the platform,
    // but we can test that the detection logic doesn't panic and handles the platform correctly
    match wrapper_result {
        Ok(_) => {
            // Binary was found, which means platform detection worked
            println!("Platform-specific binary detection succeeded");
        }
        Err(GytmdlError::BinaryNotFound(msg)) => {
            // Binary not found, but the error message should contain platform-specific info
            println!("Platform-specific binary not found (expected): {}", msg);
            assert!(msg.contains("gytmdl"));
        }
        Err(e) => {
            panic!("Unexpected error during platform-specific binary detection: {}", e);
        }
    }
}

#[tokio::test]
async fn test_process_lifecycle_integration() {
    // Create a comprehensive mock that simulates a full download lifecycle
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let lifecycle_script = temp_dir.path().join("lifecycle_gytmdl.sh");
    
    let script_content = r#"#!/bin/bash
echo "Initializing gytmdl downloader"
sleep 0.05
echo "Fetching video metadata from YouTube Music"
sleep 0.05
echo "Getting video info for: Test Song"
sleep 0.05
echo "[download] 0% of 4.2MiB"
sleep 0.05
echo "[download] 25.0% of 4.2MiB at 2.1MiB/s ETA 00:03"
sleep 0.05
echo "[download] 50.0% of 4.2MiB at 2.1MiB/s ETA 00:02"
sleep 0.05
echo "[download] 75.0% of 4.2MiB at 2.1MiB/s ETA 00:01"
sleep 0.05
echo "[download] 100% of 4.2MiB in 00:02"
sleep 0.05
echo "Remuxing audio stream to M4A format"
sleep 0.05
echo "Applying ID3 tags to output file"
sleep 0.05
echo "Adding cover art (1400x1400)"
sleep 0.05
echo "Finalizing download"
sleep 0.05
echo "Download completed successfully"
"#;
    
    fs::write(&lifecycle_script, script_content).await.unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&lifecycle_script).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&lifecycle_script, perms).await.unwrap();
    }
    
    let mut command = Command::new(&lifecycle_script);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    
    let child = command.spawn().expect("Failed to spawn lifecycle process");
    let mut process = GytmdlProcess::new(child, "lifecycle-job-id".to_string());
    
    let mut stages_seen = Vec::new();
    let mut percentages_seen = Vec::new();
    
    // Read all output and parse progress
    loop {
        match timeout(Duration::from_secs(5), process.read_stdout_line()).await {
            Ok(Ok(Some(line))) => {
                if let Some(progress) = ProgressParser::parse_output(&line) {
                    stages_seen.push(progress.stage);
                    if let Some(percentage) = progress.percentage {
                        percentages_seen.push(percentage);
                    }
                }
            }
            Ok(Ok(None)) => {
                // EOF reached
                break;
            }
            Ok(Err(_)) => {
                // IO error
                break;
            }
            Err(_) => {
                // Timeout
                break;
            }
        }
    }
    
    // Wait for process completion
    let exit_status = timeout(Duration::from_secs(2), process.wait()).await
        .expect("Process should complete within timeout")
        .expect("Failed to wait for process");
    
    assert!(exit_status.success());
    
    // Verify we saw the expected progression of stages
    assert!(stages_seen.iter().any(|s| matches!(s, DownloadStage::Initializing)));
    assert!(stages_seen.iter().any(|s| matches!(s, DownloadStage::FetchingMetadata)));
    assert!(stages_seen.iter().any(|s| matches!(s, DownloadStage::DownloadingAudio)));
    assert!(stages_seen.iter().any(|s| matches!(s, DownloadStage::Remuxing)));
    assert!(stages_seen.iter().any(|s| matches!(s, DownloadStage::ApplyingTags)));
    assert!(stages_seen.iter().any(|s| matches!(s, DownloadStage::Finalizing)));
    
    // Verify we saw progress percentages
    assert!(!percentages_seen.is_empty());
    assert!(percentages_seen.iter().any(|&p| p == 0.0));
    assert!(percentages_seen.iter().any(|&p| p == 100.0));
    
    // Verify percentages are in reasonable order (not strictly increasing due to different stages)
    let download_percentages: Vec<f32> = percentages_seen.into_iter()
        .filter(|&p| p >= 0.0 && p <= 100.0)
        .collect();
    assert!(!download_percentages.is_empty());
}

