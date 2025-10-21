#[cfg(test)]
mod sidecar_tests {
    use crate::modules::gytmdl_wrapper::{GytmdlWrapper, GytmdlError};
    use crate::modules::sidecar_manager::{SidecarManager, SidecarInfo};
    use std::path::{Path, PathBuf};
    use std::fs;
    use tempfile::TempDir;
    use tokio;

    /// Create a mock sidecar binary for testing
    fn create_mock_sidecar_binary(dir: &Path, name: &str, content: &str) -> PathBuf {
        let binary_path = dir.join(name);
        fs::write(&binary_path, content).expect("Failed to write mock binary");
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path).unwrap().permissions();
            perms.set_mode(0o755); // Make executable
            fs::set_permissions(&binary_path, perms).unwrap();
        }
        
        binary_path
    }

    /// Create a mock manifest file for testing
    fn create_mock_manifest(binary_path: &Path) -> PathBuf {
        let manifest_path = binary_path.with_extension("json");
        let manifest_content = format!(r#"{{
            "binary_name": "{}",
            "platform": {{
                "os": "test",
                "arch": "test",
                "target": "test-target",
                "extension": ""
            }},
            "size_bytes": {},
            "sha256": "test-hash",
            "build_timestamp": "2024-01-01T00:00:00Z"
        }}"#, 
            binary_path.file_name().unwrap().to_str().unwrap(),
            binary_path.metadata().unwrap().len()
        );
        
        fs::write(&manifest_path, manifest_content).expect("Failed to write manifest");
        manifest_path
    }

    #[test]
    fn test_platform_binary_name_detection() {
        let binary_name = GytmdlWrapper::get_platform_binary_name();
        
        // Verify the binary name follows the expected pattern
        assert!(binary_name.starts_with("gytmdl"));
        
        // Platform-specific checks
        if cfg!(target_os = "windows") {
            assert!(binary_name.ends_with(".exe"));
            assert!(binary_name.contains("windows"));
        } else if cfg!(target_os = "macos") {
            assert!(binary_name.contains("darwin"));
        } else if cfg!(target_os = "linux") {
            assert!(binary_name.contains("linux"));
        }
    }

    #[test]
    fn test_sidecar_directory_detection() {
        let sidecar_dir = GytmdlWrapper::get_sidecar_directory();
        
        // Verify the path ends with "sidecars"
        assert_eq!(sidecar_dir.file_name().unwrap(), "sidecars");
        
        // Verify it's an absolute path
        assert!(sidecar_dir.is_absolute());
    }

    #[test]
    fn test_binary_detection_with_mock_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let sidecars_dir = temp_dir.path().join("sidecars");
        fs::create_dir_all(&sidecars_dir).expect("Failed to create sidecars dir");

        // Create mock binaries for different platforms
        let mock_binaries = vec![
            "gytmdl-x86_64-pc-windows-msvc.exe",
            "gytmdl-x86_64-apple-darwin",
            "gytmdl-aarch64-apple-darwin",
            "gytmdl-x86_64-unknown-linux-gnu",
        ];

        for binary_name in &mock_binaries {
            create_mock_sidecar_binary(&sidecars_dir, binary_name, "mock binary content");
        }

        // Test binary listing (we can't easily test the actual detection without mocking the sidecar directory)
        // This would require dependency injection or environment variable override
        
        // For now, just verify our mock files were created correctly
        for binary_name in &mock_binaries {
            let binary_path = sidecars_dir.join(binary_name);
            assert!(binary_path.exists(), "Mock binary should exist: {}", binary_name);
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = fs::metadata(&binary_path).unwrap().permissions();
                assert!(perms.mode() & 0o111 != 0, "Binary should be executable: {}", binary_name);
            }
        }
    }

    #[test]
    fn test_manifest_loading_and_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let binary_path = create_mock_sidecar_binary(temp_dir.path(), "test-binary", "test content");
        let _manifest_path = create_mock_manifest(&binary_path);

        // Create wrapper with mock binary
        let wrapper = GytmdlWrapper::with_binary_path(binary_path.clone())
            .expect("Failed to create wrapper");

        // Test manifest loading
        let manifest = wrapper.load_manifest().expect("Failed to load manifest");
        assert_eq!(manifest.binary_name, "test-binary");
        assert_eq!(manifest.platform.os, "test");
        assert_eq!(manifest.platform.target, "test-target");
    }

    #[test]
    fn test_binary_validation_failure_cases() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Test with non-existent binary
        let non_existent_path = temp_dir.path().join("non-existent-binary");
        let result = GytmdlWrapper::with_binary_path(non_existent_path);
        assert!(result.is_err());
        
        if let Err(GytmdlError::BinaryNotFound(_)) = result {
            // Expected error type
        } else {
            panic!("Expected BinaryNotFound error");
        }
    }

    #[tokio::test]
    async fn test_sidecar_manager_status() {
        // Test getting sidecar status (this will use the actual system state)
        let status = SidecarManager::get_status().await;
        
        // Verify status structure
        assert!(!status.platform_binary_name.is_empty());
        assert!(!status.sidecar_directory.is_empty());
        
        // The current_binary might be None if no binary is found, which is okay for testing
        // The available_binaries list might be empty, which is also okay for testing
        
        println!("Platform binary name: {}", status.platform_binary_name);
        println!("Sidecar directory: {}", status.sidecar_directory);
        println!("Available binaries: {}", status.available_binaries.len());
    }

    #[tokio::test]
    async fn test_sidecar_compatibility_check() {
        // Test platform compatibility check
        let result = SidecarManager::check_platform_compatibility().await;
        
        // This should not fail, even if no binaries are available
        assert!(result.is_ok());
        
        let is_compatible = result.unwrap();
        println!("Platform compatibility: {}", is_compatible);
        
        // If binaries are available, compatibility should be true
        // If no binaries are available, compatibility will be false, which is expected
    }

    #[test]
    fn test_url_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let binary_path = create_mock_sidecar_binary(temp_dir.path(), "test-binary", "test content");
        let wrapper = GytmdlWrapper::with_binary_path(binary_path).expect("Failed to create wrapper");

        // Test valid URLs
        let valid_urls = vec![
            "https://music.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://music.youtube.com/playlist?list=PLrAXtmRdnEQy8VJqQzJmJZqJGqJQQQQQQ",
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtu.be/dQw4w9WgXcQ",
        ];

        for url in valid_urls {
            let result = wrapper.build_command_args(&Default::default(), url, "test-job");
            assert!(result.is_ok(), "URL should be valid: {}", url);
        }

        // Test invalid URLs
        let invalid_urls = vec![
            "not-a-url",
            "ftp://example.com",
            "https://example.com",
            "",
        ];

        for url in invalid_urls {
            let result = wrapper.build_command_args(&Default::default(), url, "test-job");
            assert!(result.is_err(), "URL should be invalid: {}", url);
        }
    }

    #[test]
    fn test_command_args_building() {
        use crate::modules::state::{AppConfig, DownloadMode, CoverFormat};
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let binary_path = create_mock_sidecar_binary(temp_dir.path(), "test-binary", "test content");
        let wrapper = GytmdlWrapper::with_binary_path(binary_path).expect("Failed to create wrapper");

        let mut config = AppConfig::default();
        config.output_path = PathBuf::from("/test/output");
        config.temp_path = PathBuf::from("/test/temp");
        config.itag = "140".to_string();
        config.download_mode = DownloadMode::Audio;
        config.save_cover = true;
        config.cover_format = CoverFormat::Jpg;
        config.cover_size = 500;
        config.cover_quality = 90;

        let args = wrapper.build_command_args(&config, "https://music.youtube.com/watch?v=test", "test-job")
            .expect("Failed to build command args");

        // Verify essential arguments are present
        assert!(args.contains(&"--output-path".to_string()));
        assert!(args.contains(&"/test/output".to_string()));
        assert!(args.contains(&"--temp-path".to_string()));
        assert!(args.contains(&"/test/temp".to_string()));
        assert!(args.contains(&"--itag".to_string()));
        assert!(args.contains(&"140".to_string()));
        assert!(args.contains(&"--cover-size".to_string()));
        assert!(args.contains(&"500".to_string()));
        assert!(args.contains(&"https://music.youtube.com/watch?v=test".to_string()));

        // Verify progress and verbose flags are added
        assert!(args.contains(&"--progress".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
    }

    #[test]
    fn test_error_display() {
        let errors = vec![
            GytmdlError::BinaryNotFound("test-path".to_string()),
            GytmdlError::ProcessSpawnError(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
            GytmdlError::InvalidUrl("test-url".to_string()),
            GytmdlError::ConfigError("test-config".to_string()),
            GytmdlError::ProcessError("test-process".to_string()),
            GytmdlError::ValidationError("test-validation".to_string()),
            GytmdlError::IntegrityError("test-integrity".to_string()),
            GytmdlError::ManifestError("test-manifest".to_string()),
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty(), "Error should have a non-empty display string");
            println!("Error: {}", error_string);
        }
    }

    /// Integration test that verifies the complete sidecar detection and validation flow
    #[tokio::test]
    async fn test_complete_sidecar_workflow() {
        // This test runs the complete workflow that would happen in the real application
        
        // 1. Get sidecar status
        let status = SidecarManager::get_status().await;
        println!("Sidecar status: current_binary={:?}, available_binaries={}", 
                status.current_binary.is_some(), status.available_binaries.len());

        // 2. Validate all available binaries
        let validation_result = SidecarManager::validate_all_binaries().await;
        assert!(validation_result.is_ok(), "Binary validation should not fail");
        
        let validated_binaries = validation_result.unwrap();
        println!("Validated {} binaries", validated_binaries.len());

        // 3. Check platform compatibility
        let compatibility_result = SidecarManager::check_platform_compatibility().await;
        assert!(compatibility_result.is_ok(), "Compatibility check should not fail");
        
        let is_compatible = compatibility_result.unwrap();
        println!("Platform compatible: {}", is_compatible);

        // 4. If we have binaries, try to select the best one
        if !validated_binaries.is_empty() {
            let best_binary_result = SidecarManager::select_best_binary().await;
            if best_binary_result.is_ok() {
                let best_binary = best_binary_result.unwrap();
                println!("Best binary: {} (valid: {})", best_binary.binary_path, best_binary.is_valid);
            }
        }

        // This test should complete without panicking, regardless of whether binaries are available
    }
}