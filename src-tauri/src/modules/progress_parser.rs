use crate::modules::state::{Progress, DownloadStage};
use regex::Regex;
use std::sync::OnceLock;

/// Progress parser for gytmdl output
pub struct ProgressParser;

impl ProgressParser {
    /// Parse a line of gytmdl output and extract progress information
    pub fn parse_output(output: &str) -> Option<Progress> {
        let line = output.trim();
        
        // Try different parsing strategies in order of specificity
        if let Some(progress) = Self::parse_download_progress(line) {
            return Some(progress);
        }
        
        if let Some(progress) = Self::parse_generic_progress(line) {
            return Some(progress);
        }
        
        if let Some(progress) = Self::parse_stage_indicators(line) {
            return Some(progress);
        }
        
        // Fallback: detect stage from keywords
        Self::parse_stage_from_keywords(line)
    }

    /// Parse download progress lines with percentage
    /// Examples:
    /// "[download] 45.2% of 3.45MiB at 1.23MiB/s ETA 00:02"
    /// "[download] 100% of 3.45MiB in 00:15"
    fn parse_download_progress(line: &str) -> Option<Progress> {
        static DOWNLOAD_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = DOWNLOAD_REGEX.get_or_init(|| {
            Regex::new(r"\[download\]\s+(\d+(?:\.\d+)?)%\s+of\s+[\d.]+\w+(?:\s+at\s+[\d.]+\w+/s)?(?:\s+ETA\s+[\d:]+)?(?:\s+in\s+[\d:]+)?").unwrap()
        });

        if let Some(captures) = regex.captures(line) {
            if let Some(percentage_str) = captures.get(1) {
                if let Ok(percentage) = percentage_str.as_str().parse::<f32>() {
                    return Some(Progress {
                        stage: DownloadStage::DownloadingAudio,
                        percentage: Some(percentage),
                        current_step: line.to_string(),
                        total_steps: None,
                        current_step_index: None,
                    });
                }
            }
        }
        None
    }

    /// Parse stage indicators and progress from various gytmdl output patterns
    fn parse_stage_indicators(line: &str) -> Option<Progress> {
        // Initializing/Setup patterns
        if line.contains("Initializing") || line.contains("Starting") || line.contains("Setting up") {
            return Some(Progress {
                stage: DownloadStage::Initializing,
                percentage: None,
                current_step: line.to_string(),
                total_steps: None,
                current_step_index: None,
            });
        }

        // Metadata fetching patterns
        if line.contains("Fetching") && (line.contains("metadata") || line.contains("info")) ||
           line.contains("Getting video info") || line.contains("Extracting") {
            return Some(Progress {
                stage: DownloadStage::FetchingMetadata,
                percentage: None,
                current_step: line.to_string(),
                total_steps: None,
                current_step_index: None,
            });
        }

        // Download patterns (without percentage)
        if line.contains("[download]") && !line.contains("%") {
            return Some(Progress {
                stage: DownloadStage::DownloadingAudio,
                percentage: None,
                current_step: line.to_string(),
                total_steps: None,
                current_step_index: None,
            });
        }

        // Remuxing/Processing patterns
        if line.contains("Remuxing") || line.contains("Processing") || 
           line.contains("Converting") || line.contains("Merging") {
            return Some(Progress {
                stage: DownloadStage::Remuxing,
                percentage: None,
                current_step: line.to_string(),
                total_steps: None,
                current_step_index: None,
            });
        }

        // Tagging patterns
        if line.contains("Applying tags") || line.contains("Writing tags") ||
           line.contains("Adding metadata") || line.contains("Tagging") ||
           line.contains("Writing metadata") || line.contains("Adding cover") {
            return Some(Progress {
                stage: DownloadStage::ApplyingTags,
                percentage: None,
                current_step: line.to_string(),
                total_steps: None,
                current_step_index: None,
            });
        }

        // Finalizing patterns
        if line.contains("Finalizing") || line.contains("Finishing") ||
           line.contains("Completed") || line.contains("Done") ||
           line.contains("completed") {
            return Some(Progress {
                stage: DownloadStage::Finalizing,
                percentage: None,
                current_step: line.to_string(),
                total_steps: None,
                current_step_index: None,
            });
        }

        None
    }

    /// Parse generic progress patterns with step counting
    /// Examples:
    /// "Step 3 of 5: Processing audio"
    /// "[3/5] Downloading track"
    fn parse_generic_progress(line: &str) -> Option<Progress> {
        static STEP_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = STEP_REGEX.get_or_init(|| {
            Regex::new(r"(?:Step\s+(\d+)\s+of\s+(\d+)|\[(\d+)/(\d+)\])").unwrap()
        });

        if let Some(captures) = regex.captures(line) {
            let (current, total) = if let (Some(current_str), Some(total_str)) = (captures.get(1), captures.get(2)) {
                (current_str.as_str(), total_str.as_str())
            } else if let (Some(current_str), Some(total_str)) = (captures.get(3), captures.get(4)) {
                (current_str.as_str(), total_str.as_str())
            } else {
                return None;
            };

            if let (Ok(current_step), Ok(total_steps)) = (current.parse::<u32>(), total.parse::<u32>()) {
                let percentage = if total_steps > 0 {
                    // Round to avoid floating point precision issues
                    let raw_percentage = (current_step as f32 / total_steps as f32) * 100.0;
                    Some((raw_percentage * 100.0).round() / 100.0)
                } else {
                    None
                };

                let stage = Self::infer_stage_from_step_content(line);

                return Some(Progress {
                    stage,
                    percentage,
                    current_step: line.to_string(),
                    total_steps: Some(total_steps),
                    current_step_index: Some(current_step),
                });
            }
        }
        None
    }

    /// Parse stage from keywords when no other patterns match
    fn parse_stage_from_keywords(line: &str) -> Option<Progress> {
        let lower_line = line.to_lowercase();

        let stage = if lower_line.contains("init") || lower_line.contains("start") {
            DownloadStage::Initializing
        } else if lower_line.contains("fetch") || lower_line.contains("extract") || 
                  lower_line.contains("metadata") || lower_line.contains("info") {
            DownloadStage::FetchingMetadata
        } else if lower_line.contains("download") || lower_line.contains("audio") {
            DownloadStage::DownloadingAudio
        } else if lower_line.contains("remux") || lower_line.contains("process") || 
                  lower_line.contains("convert") {
            DownloadStage::Remuxing
        } else if lower_line.contains("tag") || lower_line.contains("metadata") {
            DownloadStage::ApplyingTags
        } else if lower_line.contains("final") || lower_line.contains("complete") || 
                  lower_line.contains("done") || lower_line.contains("finish") {
            DownloadStage::Finalizing
        } else {
            // If we can't determine the stage, don't return anything
            return None;
        };

        Some(Progress {
            stage,
            percentage: None,
            current_step: line.to_string(),
            total_steps: None,
            current_step_index: None,
        })
    }

    /// Infer the download stage from step content
    fn infer_stage_from_step_content(content: &str) -> DownloadStage {
        let lower_content = content.to_lowercase();

        if lower_content.contains("init") || lower_content.contains("start") {
            DownloadStage::Initializing
        } else if lower_content.contains("fetch") || lower_content.contains("extract") || 
                  lower_content.contains("metadata") {
            DownloadStage::FetchingMetadata
        } else if lower_content.contains("download") || lower_content.contains("audio") {
            DownloadStage::DownloadingAudio
        } else if lower_content.contains("remux") || lower_content.contains("process") || 
                  lower_content.contains("convert") {
            DownloadStage::Remuxing
        } else if lower_content.contains("tag") || lower_content.contains("metadata") {
            DownloadStage::ApplyingTags
        } else if lower_content.contains("final") || lower_content.contains("complete") {
            DownloadStage::Finalizing
        } else {
            // Default to downloading if we can't determine
            DownloadStage::DownloadingAudio
        }
    }

    /// Parse error messages and return failed progress
    pub fn parse_error(error_line: &str) -> Progress {
        Progress {
            stage: DownloadStage::Failed,
            percentage: None,
            current_step: format!("Error: {}", error_line),
            total_steps: None,
            current_step_index: None,
        }
    }

    /// Create a completed progress state
    pub fn create_completed_progress() -> Progress {
        Progress {
            stage: DownloadStage::Completed,
            percentage: Some(100.0),
            current_step: "Download completed successfully".to_string(),
            total_steps: None,
            current_step_index: None,
        }
    }

    /// Create an initializing progress state
    pub fn create_initializing_progress() -> Progress {
        Progress {
            stage: DownloadStage::Initializing,
            percentage: None,
            current_step: "Initializing download...".to_string(),
            total_steps: None,
            current_step_index: None,
        }
    }

    /// Extract percentage from various percentage formats
    /// Examples: "45.2%", "45%", "0.452" (as fraction)
    fn extract_percentage(text: &str) -> Option<f32> {
        static PERCENTAGE_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = PERCENTAGE_REGEX.get_or_init(|| {
            Regex::new(r"(\d+(?:\.\d+)?)%").unwrap()
        });

        if let Some(captures) = regex.captures(text) {
            if let Some(percentage_str) = captures.get(1) {
                if let Ok(percentage) = percentage_str.as_str().parse::<f32>() {
                    return Some(percentage.clamp(0.0, 100.0));
                }
            }
        }

        // Try to parse as fraction (0.0 to 1.0)
        if let Ok(fraction) = text.parse::<f32>() {
            if fraction >= 0.0 && fraction <= 1.0 {
                return Some(fraction * 100.0);
            }
        }

        None
    }

    /// Check if a line indicates an error condition
    pub fn is_error_line(line: &str) -> bool {
        let lower_line = line.to_lowercase();
        lower_line.contains("error") || 
        lower_line.contains("failed") || 
        lower_line.contains("exception") ||
        lower_line.contains("traceback") ||
        lower_line.starts_with("error:") ||
        lower_line.starts_with("fatal:")
    }

    /// Check if a line indicates successful completion
    pub fn is_completion_line(line: &str) -> bool {
        let lower_line = line.to_lowercase();
        lower_line.contains("download completed") ||
        lower_line.contains("successfully downloaded") ||
        lower_line.contains("finished downloading") ||
        (lower_line.contains("100%") && lower_line.contains("download"))
    }

    /// Sanitize output line for display (remove ANSI codes, etc.)
    pub fn sanitize_output(line: &str) -> String {
        static ANSI_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = ANSI_REGEX.get_or_init(|| {
            Regex::new(r"\x1b\[[0-9;]*m").unwrap()
        });

        regex.replace_all(line, "").trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_download_progress() {
        let test_cases = vec![
            ("[download] 45.2% of 3.45MiB at 1.23MiB/s ETA 00:02", Some(45.2)),
            ("[download] 100% of 3.45MiB in 00:15", Some(100.0)),
            ("[download] 0% of 5.67MiB", Some(0.0)),
            ("[download] 67.8% of 2.1MiB at 500KiB/s", Some(67.8)),
            ("Not a download line", None),
        ];

        for (input, expected_percentage) in test_cases {
            let result = ProgressParser::parse_download_progress(input);
            match (result, expected_percentage) {
                (Some(progress), Some(expected)) => {
                    assert_eq!(progress.percentage, Some(expected));
                    assert!(matches!(progress.stage, DownloadStage::DownloadingAudio));
                }
                (None, None) => {
                    // Expected no match
                }
                _ => panic!("Unexpected result for input: {}", input),
            }
        }
    }

    #[test]
    fn test_parse_stage_indicators() {
        let test_cases = vec![
            ("Initializing download process", DownloadStage::Initializing),
            ("Fetching video metadata", DownloadStage::FetchingMetadata),
            ("Getting video info", DownloadStage::FetchingMetadata),
            ("[download] Destination: file.mp3", DownloadStage::DownloadingAudio),
            ("Remuxing audio stream", DownloadStage::Remuxing),
            ("Processing audio file", DownloadStage::Remuxing),
            ("Applying tags to file", DownloadStage::ApplyingTags),
            ("Writing metadata", DownloadStage::ApplyingTags),
            ("Finalizing download", DownloadStage::Finalizing),
            ("Download completed", DownloadStage::Finalizing),
        ];

        for (input, expected_stage) in test_cases {
            let result = ProgressParser::parse_stage_indicators(input);
            assert!(result.is_some(), "Expected match for: {}", input);
            let progress = result.unwrap();
            assert!(matches!(progress.stage, expected_stage), 
                   "Expected {:?} for input: {}", expected_stage, input);
        }
    }

    #[test]
    fn test_parse_generic_progress() {
        let test_cases = vec![
            ("Step 3 of 5: Processing audio", Some((3, 5, 60.0))),
            ("[2/4] Downloading track", Some((2, 4, 50.0))),
            ("Step 1 of 1: Complete", Some((1, 1, 100.0))),
            ("No step info here", None),
        ];

        for (input, expected) in test_cases {
            let result = ProgressParser::parse_generic_progress(input);
            match (result, expected) {
                (Some(progress), Some((current, total, expected_percentage))) => {
                    assert_eq!(progress.current_step_index, Some(current));
                    assert_eq!(progress.total_steps, Some(total));
                    // Use approximate comparison for floating point values
                    if let Some(actual_percentage) = progress.percentage {
                        assert!((actual_percentage - expected_percentage).abs() < 0.01, 
                               "Expected percentage ~{}, got {}", expected_percentage, actual_percentage);
                    } else {
                        panic!("Expected percentage {}, got None", expected_percentage);
                    }
                }
                (None, None) => {
                    // Expected no match
                }
                _ => panic!("Unexpected result for input: {}", input),
            }
        }
    }

    #[test]
    fn test_parse_stage_from_keywords() {
        let test_cases = vec![
            ("Starting process", Some(DownloadStage::Initializing)),
            ("Extracting information", Some(DownloadStage::FetchingMetadata)),
            ("Downloading audio stream", Some(DownloadStage::DownloadingAudio)),
            ("Converting format", Some(DownloadStage::Remuxing)),
            ("Adding tags", Some(DownloadStage::ApplyingTags)),
            ("Finalizing output", Some(DownloadStage::Finalizing)),
            ("Random text with no keywords", None),
        ];

        for (input, expected) in test_cases {
            let result = ProgressParser::parse_stage_from_keywords(input);
            match (result, expected) {
                (Some(progress), Some(expected_stage)) => {
                    assert!(matches!(progress.stage, expected_stage), 
                           "Expected {:?} for input: {}", expected_stage, input);
                }
                (None, None) => {
                    // Expected no match
                }
                _ => panic!("Unexpected result for input: {}", input),
            }
        }
    }

    #[test]
    fn test_extract_percentage() {
        let test_cases = vec![
            ("45.2%", Some(45.2)),
            ("100%", Some(100.0)),
            ("0%", Some(0.0)),
            ("0.452", Some(45.2)),
            ("1.0", Some(100.0)),
            ("0.0", Some(0.0)),
            ("150%", Some(100.0)), // Should be clamped
            ("no percentage here", None),
        ];

        for (input, expected) in test_cases {
            let result = ProgressParser::extract_percentage(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_is_error_line() {
        let error_lines = vec![
            "Error: Failed to download",
            "ERROR: Network timeout",
            "Exception occurred during processing",
            "Fatal: Cannot continue",
            "Download failed with error code 1",
        ];

        let normal_lines = vec![
            "[download] 50% complete",
            "Processing audio file",
            "Download completed successfully",
        ];

        for line in error_lines {
            assert!(ProgressParser::is_error_line(line), "Should detect error: {}", line);
        }

        for line in normal_lines {
            assert!(!ProgressParser::is_error_line(line), "Should not detect error: {}", line);
        }
    }

    #[test]
    fn test_is_completion_line() {
        let completion_lines = vec![
            "Download completed successfully",
            "Successfully downloaded track.mp3",
            "Finished downloading album",
            "[download] 100% of 5.0MiB",
        ];

        let normal_lines = vec![
            "[download] 50% complete",
            "Processing audio file",
            "Starting download",
        ];

        for line in completion_lines {
            assert!(ProgressParser::is_completion_line(line), "Should detect completion: {}", line);
        }

        for line in normal_lines {
            assert!(!ProgressParser::is_completion_line(line), "Should not detect completion: {}", line);
        }
    }

    #[test]
    fn test_sanitize_output() {
        let test_cases = vec![
            ("\x1b[32mGreen text\x1b[0m", "Green text"),
            ("\x1b[1;31mBold red\x1b[0m", "Bold red"),
            ("Normal text", "Normal text"),
            ("  \x1b[33mYellow\x1b[0m  ", "Yellow"),
        ];

        for (input, expected) in test_cases {
            let result = ProgressParser::sanitize_output(input);
            assert_eq!(result, expected, "Failed for input: {:?}", input);
        }
    }

    #[test]
    fn test_parse_output_integration() {
        let test_cases = vec![
            ("[download] 75.5% of 4.2MiB at 2.1MiB/s ETA 00:01", DownloadStage::DownloadingAudio, Some(75.5)),
            ("Fetching video metadata from YouTube", DownloadStage::FetchingMetadata, None),
            ("Step 2 of 4: Converting audio format", DownloadStage::Remuxing, Some(50.0)),
            ("Applying ID3 tags to output file", DownloadStage::ApplyingTags, None),
            ("Download completed successfully", DownloadStage::Finalizing, None),
        ];

        for (input, expected_stage, expected_percentage) in test_cases {
            let result = ProgressParser::parse_output(input);
            assert!(result.is_some(), "Should parse: {}", input);
            
            let progress = result.unwrap();
            assert!(matches!(progress.stage, expected_stage), 
                   "Expected stage {:?} for: {}", expected_stage, input);
            
            // Use approximate comparison for floating point values
            match (progress.percentage, expected_percentage) {
                (Some(actual), Some(expected)) => {
                    assert!((actual - expected).abs() < 0.01, 
                           "Expected percentage ~{}, got {} for: {}", expected, actual, input);
                }
                (None, None) => {
                    // Both None, this is expected
                }
                (actual, expected) => {
                    panic!("Expected percentage {:?}, got {:?} for: {}", expected, actual, input);
                }
            }
            assert_eq!(progress.current_step, input);
        }
    }

    #[test]
    fn test_create_progress_states() {
        let completed = ProgressParser::create_completed_progress();
        assert!(matches!(completed.stage, DownloadStage::Completed));
        assert_eq!(completed.percentage, Some(100.0));

        let initializing = ProgressParser::create_initializing_progress();
        assert!(matches!(initializing.stage, DownloadStage::Initializing));
        assert!(initializing.percentage.is_none());

        let error = ProgressParser::parse_error("Network connection failed");
        assert!(matches!(error.stage, DownloadStage::Failed));
        assert!(error.current_step.contains("Error:"));
    }
}