#[cfg(test)]
mod packaging_tests {
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    /// Test that verifies the Tauri configuration is valid
    #[test]
    fn test_tauri_config_validity() {
        let config_path = PathBuf::from("src-tauri/tauri.conf.json");
        assert!(config_path.exists(), "Tauri config file should exist");

        // Parse the config to ensure it's valid JSON
        let config_content = fs::read_to_string(&config_path)
            .expect("Failed to read tauri.conf.json");
        
        let config: serde_json::Value = serde_json::from_str(&config_content)
            .expect("tauri.conf.json should be valid JSON");

        // Verify essential configuration sections
        assert!(config.get("productName").is_some(), "productName should be defined");
        assert!(config.get("version").is_some(), "version should be defined");
        assert!(config.get("identifier").is_some(), "identifier should be defined");
        assert!(config.get("bundle").is_some(), "bundle configuration should be defined");

        // Verify bundle configuration
        let bundle = config.get("bundle").unwrap();
        assert!(bundle.get("active").is_some(), "bundle.active should be defined");
        assert!(bundle.get("targets").is_some(), "bundle.targets should be defined");
        assert!(bundle.get("icon").is_some(), "bundle.icon should be defined");
        assert!(bundle.get("resources").is_some(), "bundle.resources should be defined");
        assert!(bundle.get("externalBin").is_some(), "bundle.externalBin should be defined");

        // Verify sidecar binaries are configured
        let external_bin = bundle.get("externalBin").unwrap().as_array()
            .expect("externalBin should be an array");
        
        assert!(!external_bin.is_empty(), "externalBin should not be empty");
        
        // Check that expected sidecar binaries are listed
        let external_bin_strings: Vec<String> = external_bin.iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        
        assert!(external_bin_strings.iter().any(|s| s.contains("windows-msvc")), 
                "Windows sidecar should be configured");
        assert!(external_bin_strings.iter().any(|s| s.contains("apple-darwin")), 
                "macOS sidecar should be configured");
        assert!(external_bin_strings.iter().any(|s| s.contains("linux-gnu")), 
                "Linux sidecar should be configured");
    }

    /// Test that verifies the package.json is properly configured
    #[test]
    fn test_package_json_validity() {
        let package_path = PathBuf::from("package.json");
        assert!(package_path.exists(), "package.json should exist");

        let package_content = fs::read_to_string(&package_path)
            .expect("Failed to read package.json");
        
        let package: serde_json::Value = serde_json::from_str(&package_content)
            .expect("package.json should be valid JSON");

        // Verify essential fields
        assert!(package.get("name").is_some(), "name should be defined");
        assert!(package.get("version").is_some(), "version should be defined");
        assert!(package.get("scripts").is_some(), "scripts should be defined");
        assert!(package.get("devDependencies").is_some(), "devDependencies should be defined");

        // Verify Tauri-related scripts
        let scripts = package.get("scripts").unwrap();
        assert!(scripts.get("tauri").is_some(), "tauri script should be defined");
        assert!(scripts.get("build").is_some(), "build script should be defined");
        assert!(scripts.get("dev").is_some(), "dev script should be defined");

        // Verify Tauri is in devDependencies
        let dev_deps = package.get("devDependencies").unwrap();
        assert!(dev_deps.get("@tauri-apps/cli").is_some(), "Tauri CLI should be in devDependencies");
    }

    /// Test that verifies Cargo.toml is properly configured for Tauri
    #[test]
    fn test_cargo_toml_validity() {
        let cargo_path = PathBuf::from("src-tauri/Cargo.toml");
        assert!(cargo_path.exists(), "Cargo.toml should exist");

        let cargo_content = fs::read_to_string(&cargo_path)
            .expect("Failed to read Cargo.toml");

        // Basic validation - check for essential sections
        assert!(cargo_content.contains("[package]"), "Cargo.toml should have [package] section");
        assert!(cargo_content.contains("[dependencies]"), "Cargo.toml should have [dependencies] section");
        assert!(cargo_content.contains("tauri"), "Cargo.toml should include tauri dependency");
        assert!(cargo_content.contains("serde"), "Cargo.toml should include serde dependency");
        assert!(cargo_content.contains("tokio"), "Cargo.toml should include tokio dependency");

        // Check for our custom dependencies
        assert!(cargo_content.contains("which"), "Cargo.toml should include which dependency for binary detection");
        assert!(cargo_content.contains("regex"), "Cargo.toml should include regex dependency");
        assert!(cargo_content.contains("chrono"), "Cargo.toml should include chrono dependency");
        assert!(cargo_content.contains("uuid"), "Cargo.toml should include uuid dependency");
    }

    /// Test that verifies build scripts exist and are executable
    #[test]
    fn test_build_scripts_exist() {
        let scripts_to_check = vec![
            "build-scripts/build-sidecars.py",
            "build-scripts/build-all-platforms.sh",
            "build-scripts/build-all-platforms.bat",
            "scripts/build-and-package.py",
            "scripts/dev-build.sh",
        ];

        for script_path in scripts_to_check {
            let path = PathBuf::from(script_path);
            assert!(path.exists(), "Build script should exist: {}", script_path);

            // Check if shell scripts are executable (Unix only)
            #[cfg(unix)]
            if script_path.ends_with(".sh") {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&path).expect("Failed to get script metadata");
                let permissions = metadata.permissions();
                assert!(permissions.mode() & 0o111 != 0, 
                        "Shell script should be executable: {}", script_path);
            }
        }
    }

    /// Test that verifies PyInstaller spec file is valid
    #[test]
    fn test_pyinstaller_spec_validity() {
        let spec_path = PathBuf::from("build-scripts/pyinstaller-config.spec");
        assert!(spec_path.exists(), "PyInstaller spec file should exist");

        let spec_content = fs::read_to_string(&spec_path)
            .expect("Failed to read PyInstaller spec file");

        // Check for essential PyInstaller components
        assert!(spec_content.contains("Analysis"), "Spec should contain Analysis");
        assert!(spec_content.contains("PYZ"), "Spec should contain PYZ");
        assert!(spec_content.contains("EXE"), "Spec should contain EXE");
        assert!(spec_content.contains("gytmdl"), "Spec should reference gytmdl");
        
        // Check for our custom configuration
        assert!(spec_content.contains("platform_suffix"), "Spec should handle platform suffixes");
        assert!(spec_content.contains("hiddenimports"), "Spec should define hidden imports");
    }

    /// Test that verifies installer configuration files exist
    #[test]
    fn test_installer_configs_exist() {
        let configs_to_check = vec![
            ("src-tauri/entitlements.plist", "macOS entitlements"),
            ("src-tauri/wix-template.wxs", "Windows WiX template"),
            ("build-config.json", "Build configuration"),
        ];

        for (config_path, description) in configs_to_check {
            let path = PathBuf::from(config_path);
            assert!(path.exists(), "{} should exist: {}", description, config_path);
        }
    }

    /// Test that verifies Linux maintainer scripts exist and are executable
    #[test]
    fn test_linux_maintainer_scripts() {
        let scripts = vec![
            "src-tauri/scripts/preinst.sh",
            "src-tauri/scripts/postinst.sh", 
            "src-tauri/scripts/prerm.sh",
            "src-tauri/scripts/postrm.sh",
        ];

        for script_path in scripts {
            let path = PathBuf::from(script_path);
            assert!(path.exists(), "Linux maintainer script should exist: {}", script_path);

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&path).expect("Failed to get script metadata");
                let permissions = metadata.permissions();
                assert!(permissions.mode() & 0o111 != 0, 
                        "Maintainer script should be executable: {}", script_path);
            }

            // Verify script has proper shebang
            let content = fs::read_to_string(&path)
                .expect("Failed to read maintainer script");
            assert!(content.starts_with("#!/bin/bash"), 
                    "Maintainer script should have bash shebang: {}", script_path);
        }
    }

    /// Test that verifies GitHub Actions workflow is properly configured
    #[test]
    fn test_github_actions_workflow() {
        let workflow_path = PathBuf::from(".github/workflows/build-and-release.yml");
        assert!(workflow_path.exists(), "GitHub Actions workflow should exist");

        let workflow_content = fs::read_to_string(&workflow_path)
            .expect("Failed to read GitHub Actions workflow");

        // Check for essential workflow components
        assert!(workflow_content.contains("name: Build and Release"), 
                "Workflow should have proper name");
        assert!(workflow_content.contains("build-sidecars"), 
                "Workflow should include sidecar build job");
        assert!(workflow_content.contains("build-tauri"), 
                "Workflow should include Tauri build job");
        assert!(workflow_content.contains("create-release"), 
                "Workflow should include release job");
        assert!(workflow_content.contains("test-installers"), 
                "Workflow should include installer testing");

        // Check for platform matrix
        assert!(workflow_content.contains("macos-latest"), 
                "Workflow should include macOS builds");
        assert!(workflow_content.contains("ubuntu-20.04"), 
                "Workflow should include Linux builds");
        assert!(workflow_content.contains("windows-latest"), 
                "Workflow should include Windows builds");

        // Check for target architectures
        assert!(workflow_content.contains("x86_64-apple-darwin"), 
                "Workflow should target Intel macOS");
        assert!(workflow_content.contains("aarch64-apple-darwin"), 
                "Workflow should target Apple Silicon macOS");
        assert!(workflow_content.contains("x86_64-unknown-linux-gnu"), 
                "Workflow should target Linux x64");
        assert!(workflow_content.contains("x86_64-pc-windows-msvc"), 
                "Workflow should target Windows x64");
    }

    /// Test that simulates the build configuration loading
    #[test]
    fn test_build_config_loading() {
        let config_path = PathBuf::from("build-config.json");
        assert!(config_path.exists(), "Build config should exist");

        let config_content = fs::read_to_string(&config_path)
            .expect("Failed to read build config");
        
        let config: serde_json::Value = serde_json::from_str(&config_content)
            .expect("Build config should be valid JSON");

        // Verify essential configuration sections
        assert!(config.get("release").is_some(), "release setting should be defined");
        assert!(config.get("code_signing").is_some(), "code_signing section should be defined");
        assert!(config.get("bundle").is_some(), "bundle section should be defined");

        // Verify platform-specific configurations
        let bundle = config.get("bundle").unwrap();
        assert!(bundle.get("windows").is_some(), "Windows bundle config should be defined");
        assert!(bundle.get("macos").is_some(), "macOS bundle config should be defined");
        assert!(bundle.get("linux").is_some(), "Linux bundle config should be defined");

        // Verify code signing structure (even if disabled)
        let code_signing = config.get("code_signing").unwrap();
        assert!(code_signing.get("enabled").is_some(), "code_signing.enabled should be defined");
        assert!(code_signing.get("windows").is_some(), "Windows signing config should be defined");
        assert!(code_signing.get("macos").is_some(), "macOS signing config should be defined");
    }

    /// Test that verifies the sidecar directory structure
    #[test]
    fn test_sidecar_directory_structure() {
        let sidecars_dir = PathBuf::from("src-tauri/sidecars");
        
        // The directory might not exist yet (created during build), so we just verify
        // that the parent directory exists
        let parent_dir = sidecars_dir.parent().unwrap();
        assert!(parent_dir.exists(), "src-tauri directory should exist");

        // If the sidecars directory exists, verify it's properly structured
        if sidecars_dir.exists() {
            assert!(sidecars_dir.is_dir(), "sidecars should be a directory");
            
            // Check for any existing sidecar binaries
            if let Ok(entries) = fs::read_dir(&sidecars_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let filename = path.file_name().unwrap().to_str().unwrap();
                        
                        if filename.starts_with("gytmdl") && !filename.ends_with(".json") {
                            println!("Found sidecar binary: {}", filename);
                            
                            // Verify corresponding manifest exists
                            let manifest_path = path.with_extension("json");
                            if manifest_path.exists() {
                                println!("Found manifest: {}", manifest_path.file_name().unwrap().to_str().unwrap());
                                
                                // Verify manifest is valid JSON
                                let manifest_content = fs::read_to_string(&manifest_path)
                                    .expect("Failed to read manifest");
                                let _manifest: serde_json::Value = serde_json::from_str(&manifest_content)
                                    .expect("Manifest should be valid JSON");
                            }
                        }
                    }
                }
            }
        }
    }

    /// Test that simulates running the build pipeline validation
    #[test]
    fn test_build_pipeline_validation() {
        // This test validates that all the components needed for the build pipeline are in place
        
        // Check for required tools (this will vary by system, so we make it informational)
        let tools_to_check = vec!["node", "npm", "cargo", "python3"];
        
        for tool in tools_to_check {
            match Command::new(tool).arg("--version").output() {
                Ok(output) => {
                    if output.status.success() {
                        println!("✓ {} is available", tool);
                    } else {
                        println!("⚠ {} returned non-zero exit code", tool);
                    }
                }
                Err(_) => {
                    println!("⚠ {} is not available in PATH", tool);
                }
            }
        }

        // Verify project structure
        let required_dirs = vec![
            "src",
            "src-tauri",
            "src-tauri/src",
            "build-scripts",
            "scripts",
        ];

        for dir in required_dirs {
            let path = PathBuf::from(dir);
            assert!(path.exists() && path.is_dir(), "Required directory should exist: {}", dir);
        }

        // Verify essential files
        let required_files = vec![
            "package.json",
            "src-tauri/Cargo.toml",
            "src-tauri/tauri.conf.json",
            "build-config.json",
        ];

        for file in required_files {
            let path = PathBuf::from(file);
            assert!(path.exists() && path.is_file(), "Required file should exist: {}", file);
        }

        println!("✓ Build pipeline validation completed");
    }
}