#[cfg(test)]
mod cross_platform_tests {
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::env;

    /// Test that verifies platform-specific binary naming conventions
    #[test]
    fn test_platform_binary_naming() {
        use crate::modules::gytmdl_wrapper::GytmdlWrapper;
        
        let binary_name = GytmdlWrapper::get_platform_binary_name();
        
        // All platform binaries should start with "gytmdl"
        assert!(binary_name.starts_with("gytmdl"), 
                "Binary name should start with 'gytmdl': {}", binary_name);
        
        // Platform-specific validation
        if cfg!(target_os = "windows") {
            assert!(binary_name.ends_with(".exe"), 
                    "Windows binary should end with .exe: {}", binary_name);
            assert!(binary_name.contains("windows") || binary_name.contains("msvc"), 
                    "Windows binary should contain platform identifier: {}", binary_name);
        } else if cfg!(target_os = "macos") {
            assert!(!binary_name.ends_with(".exe"), 
                    "macOS binary should not end with .exe: {}", binary_name);
            assert!(binary_name.contains("darwin") || binary_name.contains("apple"), 
                    "macOS binary should contain platform identifier: {}", binary_name);
            
            // Architecture-specific checks
            if cfg!(target_arch = "aarch64") {
                assert!(binary_name.contains("aarch64"), 
                        "Apple Silicon binary should contain aarch64: {}", binary_name);
            } else {
                assert!(binary_name.contains("x86_64"), 
                        "Intel Mac binary should contain x86_64: {}", binary_name);
            }
        } else if cfg!(target_os = "linux") {
            assert!(!binary_name.ends_with(".exe"), 
                    "Linux binary should not end with .exe: {}", binary_name);
            assert!(binary_name.contains("linux") || binary_name.contains("gnu"), 
                    "Linux binary should contain platform identifier: {}", binary_name);
        }
        
        println!("Platform binary name: {}", binary_name);
    }

    /// Test that verifies path handling works across platforms
    #[test]
    fn test_cross_platform_path_handling() {
        use crate::modules::gytmdl_wrapper::GytmdlWrapper;
        
        let sidecar_dir = GytmdlWrapper::get_sidecar_directory();
        
        // Should be an absolute path on all platforms
        assert!(sidecar_dir.is_absolute(), 
                "Sidecar directory should be absolute: {}", sidecar_dir.display());
        
        // Should end with "sidecars"
        assert_eq!(sidecar_dir.file_name().unwrap(), "sidecars",
                   "Sidecar directory should end with 'sidecars': {}", sidecar_dir.display());
        
        // Path should be valid for the current platform
        let path_str = sidecar_dir.to_string_lossy();
        
        if cfg!(target_os = "windows") {
            // Windows paths might start with drive letter or UNC
            assert!(path_str.len() > 3, "Windows path should be substantial: {}", path_str);
        } else {
            // Unix-like paths should start with /
            assert!(path_str.starts_with('/'), "Unix path should start with /: {}", path_str);
        }
        
        println!("Sidecar directory: {}", sidecar_dir.display());
    }

    /// Test that verifies file permissions are handled correctly across platforms
    #[test]
    fn test_cross_platform_permissions() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test_binary");
        
        // Create a test file
        fs::write(&test_file, "test content").expect("Failed to write test file");
        
        // Test permission handling
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            
            // Make file executable
            let mut perms = fs::metadata(&test_file).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&test_file, perms).unwrap();
            
            // Verify it's executable
            let updated_perms = fs::metadata(&test_file).unwrap().permissions();
            assert!(updated_perms.mode() & 0o111 != 0, "File should be executable on Unix");
        }
        
        #[cfg(windows)]
        {
            // On Windows, we can't easily test execute permissions in the same way
            // but we can verify the file exists and is readable
            assert!(test_file.exists(), "File should exist on Windows");
            let metadata = fs::metadata(&test_file).unwrap();
            assert!(!metadata.permissions().readonly(), "File should not be readonly");
        }
        
        println!("Permission test completed for current platform");
    }

    /// Test that verifies environment variable handling across platforms
    #[test]
    fn test_cross_platform_environment() {
        // Test PATH separator
        let path_separator = if cfg!(target_os = "windows") { ";" } else { ":" };
        
        if let Ok(path_env) = env::var("PATH") {
            assert!(path_env.contains(path_separator), 
                    "PATH should contain platform-appropriate separator");
            println!("PATH separator: {}", path_separator);
        }
        
        // Test current directory
        let current_dir = env::current_dir().expect("Should be able to get current directory");
        assert!(current_dir.is_absolute(), "Current directory should be absolute");
        
        // Test executable extension
        let exe_extension = env::consts::EXE_EXTENSION;
        if cfg!(target_os = "windows") {
            assert_eq!(exe_extension, "exe", "Windows should have .exe extension");
        } else {
            assert_eq!(exe_extension, "", "Unix should have no extension");
        }
        
        println!("Executable extension: '{}'", exe_extension);
        println!("Current directory: {}", current_dir.display());
    }

    /// Test that verifies command line argument handling across platforms
    #[test]
    fn test_cross_platform_command_args() {
        use crate::modules::gytmdl_wrapper::GytmdlWrapper;
        use crate::modules::state::AppConfig;
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let binary_path = temp_dir.path().join("test_binary");
        fs::write(&binary_path, "test").expect("Failed to write test binary");
        
        let wrapper = GytmdlWrapper::with_binary_path(binary_path)
            .expect("Failed to create wrapper");
        
        let mut config = AppConfig::default();
        
        // Use platform-appropriate paths
        if cfg!(target_os = "windows") {
            config.output_path = PathBuf::from("C:\\test\\output");
            config.temp_path = PathBuf::from("C:\\test\\temp");
        } else {
            config.output_path = PathBuf::from("/test/output");
            config.temp_path = PathBuf::from("/test/temp");
        }
        
        let args = wrapper.build_command_args(&config, "https://music.youtube.com/watch?v=test", "test-job")
            .expect("Failed to build command args");
        
        // Verify paths are properly formatted for the platform
        let output_path_str = config.output_path.to_string_lossy();
        let temp_path_str = config.temp_path.to_string_lossy();
        
        assert!(args.contains(&output_path_str.to_string()), 
                "Args should contain output path: {}", output_path_str);
        assert!(args.contains(&temp_path_str.to_string()), 
                "Args should contain temp path: {}", temp_path_str);
        
        // Verify URL is preserved correctly
        assert!(args.contains(&"https://music.youtube.com/watch?v=test".to_string()),
                "Args should contain the URL");
        
        println!("Command args test completed for current platform");
    }

    /// Test that verifies file system operations work across platforms
    #[test]
    fn test_cross_platform_file_operations() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Test directory creation
        let test_subdir = temp_dir.path().join("test_subdir");
        fs::create_dir_all(&test_subdir).expect("Should be able to create directory");
        assert!(test_subdir.exists() && test_subdir.is_dir(), "Directory should exist");
        
        // Test file creation with various names
        let test_files = vec![
            "simple_file.txt",
            "file-with-dashes.txt",
            "file_with_underscores.txt",
            "file.with.dots.txt",
        ];
        
        for filename in test_files {
            let file_path = test_subdir.join(filename);
            fs::write(&file_path, "test content").expect("Should be able to write file");
            assert!(file_path.exists() && file_path.is_file(), "File should exist: {}", filename);
            
            // Test reading
            let content = fs::read_to_string(&file_path).expect("Should be able to read file");
            assert_eq!(content, "test content", "File content should match");
        }
        
        // Test path manipulation
        let complex_path = test_subdir.join("subdir").join("file.txt");
        fs::create_dir_all(complex_path.parent().unwrap()).expect("Should create nested dirs");
        fs::write(&complex_path, "nested content").expect("Should write to nested file");
        assert!(complex_path.exists(), "Nested file should exist");
        
        println!("File operations test completed for current platform");
    }

    /// Test that verifies process spawning works across platforms
    #[test]
    fn test_cross_platform_process_spawning() {
        use std::process::Command;
        
        // Test basic command that should work on all platforms
        let (cmd, args) = if cfg!(target_os = "windows") {
            ("cmd", vec!["/C", "echo", "test"])
        } else {
            ("echo", vec!["test"])
        };
        
        let output = Command::new(cmd)
            .args(&args)
            .output()
            .expect("Should be able to spawn process");
        
        assert!(output.status.success(), "Command should succeed");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("test"), "Output should contain 'test'");
        
        println!("Process spawning test completed for current platform");
    }

    /// Test that verifies URL handling is consistent across platforms
    #[test]
    fn test_cross_platform_url_handling() {
        use crate::modules::gytmdl_wrapper::GytmdlWrapper;
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let binary_path = temp_dir.path().join("test_binary");
        fs::write(&binary_path, "test").expect("Failed to write test binary");
        
        let wrapper = GytmdlWrapper::with_binary_path(binary_path)
            .expect("Failed to create wrapper");
        
        // Test various URL formats that should work on all platforms
        let test_urls = vec![
            "https://music.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://music.youtube.com/playlist?list=PLtest",
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtu.be/dQw4w9WgXcQ",
        ];
        
        for url in test_urls {
            let result = wrapper.build_command_args(&Default::default(), url, "test-job");
            assert!(result.is_ok(), "URL should be valid on all platforms: {}", url);
            
            let args = result.unwrap();
            assert!(args.contains(&url.to_string()), "Args should contain the URL: {}", url);
        }
        
        // Test invalid URLs that should fail on all platforms
        let invalid_urls = vec![
            "not-a-url",
            "ftp://example.com",
            "https://example.com",
            "",
            "file:///local/file",
        ];
        
        for url in invalid_urls {
            let result = wrapper.build_command_args(&Default::default(), url, "test-job");
            assert!(result.is_err(), "URL should be invalid on all platforms: {}", url);
        }
        
        println!("URL handling test completed for current platform");
    }

    /// Test that verifies configuration serialization works across platforms
    #[test]
    fn test_cross_platform_config_serialization() {
        use crate::modules::state::{AppConfig, DownloadMode, CoverFormat};
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create a config with platform-appropriate paths
        let mut config = AppConfig::default();
        
        if cfg!(target_os = "windows") {
            config.output_path = PathBuf::from("C:\\Users\\Test\\Downloads");
            config.temp_path = PathBuf::from("C:\\Temp");
            config.cookies_path = Some(PathBuf::from("C:\\Users\\Test\\cookies.txt"));
        } else {
            config.output_path = PathBuf::from("/home/test/Downloads");
            config.temp_path = PathBuf::from("/tmp");
            config.cookies_path = Some(PathBuf::from("/home/test/cookies.txt"));
        }
        
        config.download_mode = DownloadMode::Audio;
        config.cover_format = CoverFormat::Jpg;
        config.save_cover = true;
        
        // Test serialization
        let serialized = serde_json::to_string(&config)
            .expect("Should be able to serialize config");
        
        // Test deserialization
        let deserialized: AppConfig = serde_json::from_str(&serialized)
            .expect("Should be able to deserialize config");
        
        // Verify paths are preserved correctly
        assert_eq!(config.output_path, deserialized.output_path, "Output path should be preserved");
        assert_eq!(config.temp_path, deserialized.temp_path, "Temp path should be preserved");
        assert_eq!(config.cookies_path, deserialized.cookies_path, "Cookies path should be preserved");
        assert_eq!(config.download_mode, deserialized.download_mode, "Download mode should be preserved");
        assert_eq!(config.cover_format, deserialized.cover_format, "Cover format should be preserved");
        
        // Test writing to file
        let config_file = temp_dir.path().join("test_config.json");
        fs::write(&config_file, &serialized).expect("Should be able to write config file");
        
        // Test reading from file
        let file_content = fs::read_to_string(&config_file)
            .expect("Should be able to read config file");
        let file_config: AppConfig = serde_json::from_str(&file_content)
            .expect("Should be able to parse config from file");
        
        assert_eq!(config.output_path, file_config.output_path, "File config should match");
        
        println!("Config serialization test completed for current platform");
    }

    /// Test that verifies error handling is consistent across platforms
    #[test]
    fn test_cross_platform_error_handling() {
        use crate::modules::gytmdl_wrapper::{GytmdlWrapper, GytmdlError};
        
        // Test with non-existent binary
        let non_existent = PathBuf::from("definitely_does_not_exist");
        let result = GytmdlWrapper::with_binary_path(non_existent);
        
        assert!(result.is_err(), "Should fail with non-existent binary");
        
        match result.unwrap_err() {
            GytmdlError::BinaryNotFound(path) => {
                assert!(path.contains("definitely_does_not_exist"), 
                        "Error should contain the path: {}", path);
            }
            other => panic!("Expected BinaryNotFound error, got: {:?}", other),
        }
        
        // Test error display
        let errors = vec![
            GytmdlError::BinaryNotFound("test-path".to_string()),
            GytmdlError::InvalidUrl("test-url".to_string()),
            GytmdlError::ConfigError("test-config".to_string()),
            GytmdlError::ProcessError("test-process".to_string()),
            GytmdlError::ValidationError("test-validation".to_string()),
            GytmdlError::IntegrityError("test-integrity".to_string()),
            GytmdlError::ManifestError("test-manifest".to_string()),
        ];
        
        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty(), "Error should have display string");
            assert!(error_string.len() > 10, "Error string should be descriptive");
            println!("Error: {}", error_string);
        }
        
        println!("Error handling test completed for current platform");
    }
}