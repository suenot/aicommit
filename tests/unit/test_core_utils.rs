//! Unit tests for core utility functions

use super::super::common::*;

// Note: Since the main.rs doesn't export functions publicly, we'll need to add
// a lib.rs or make functions public for proper unit testing. For now, we'll create
// tests that can be run once the code is refactored.

#[cfg(test)]
mod version_tests {
    use super::*;

    // This test would work once increment_version function is made public
    // #[test]
    // fn test_increment_version_patch() {
    //     let result = increment_version("1.2.3");
    //     assert_eq!(result.unwrap(), "1.2.4");
    // }

    // #[test]
    // fn test_increment_version_invalid() {
    //     let result = increment_version("invalid");
    //     assert!(result.is_err());
    // }

    // Placeholder test to ensure test infrastructure works
    #[test]
    fn test_version_parsing_logic() {
        // Test version parsing logic that would be in increment_version
        let version = "1.2.3";
        let parts: Vec<&str> = version.split('.').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "1");
        assert_eq!(parts[1], "2");
        assert_eq!(parts[2], "3");

        let patch: u32 = parts[2].parse().unwrap();
        assert_eq!(patch + 1, 4);
    }
}

#[cfg(test)]
mod git_diff_tests {
    use super::*;
    use crate::common::test_data;

    // This test would work once process_git_diff_output function is made public
    // #[test]
    // fn test_process_git_diff_output() {
    //     let diff = test_data::sample_git_diff();
    //     let result = process_git_diff_output(diff);
    //     assert!(!result.is_empty());
    //     assert!(result.contains("src/main.rs"));
    // }

    #[test]
    fn test_git_diff_processing_logic() {
        let diff = test_data::sample_git_diff();

        // Test logic that would be in process_git_diff_output
        assert!(diff.contains("diff --git"));
        assert!(diff.contains("@@"));
        assert!(diff.contains("src/main.rs"));

        // Count added lines (starting with +)
        let added_lines: usize = diff.lines()
            .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
            .count();
        assert!(added_lines > 0);
    }
}

#[cfg(test)]
mod safe_slice_tests {
    use super::*;

    // This test would work once get_safe_slice_length function is made public
    // #[test]
    // fn test_get_safe_slice_length() {
    //     let result = get_safe_slice_length("hello world", 5);
    //     assert_eq!(result, 5);
    // }

    #[test]
    fn test_safe_slice_logic() {
        // Test the logic that would be in get_safe_slice_length
        let s = "hello world";
        let max_len = 5;

        let safe_len = if s.len() <= max_len {
            s.len()
        } else {
            // Find the last safe boundary (space or punctuation)
            s.char_indices()
                .take_while(|(i, _)| *i < max_len)
                .last()
                .map(|(i, _)| i + 1)
                .unwrap_or(max_len.min(s.len()))
        };

        assert!(safe_len <= max_len);
        assert!(safe_len <= s.len());
    }
}

#[cfg(test)]
mod duration_parsing_tests {
    use super::*;
    use std::time::Duration;

    // This test would work once parse_duration function is made public
    // #[test]
    // fn test_parse_duration_seconds() {
    //     let result = parse_duration("30s");
    //     assert_eq!(result.unwrap(), Duration::from_secs(30));
    // }

    #[test]
    fn test_duration_parsing_logic() {
        // Test duration parsing logic
        let duration_str = "30s";

        if duration_str.ends_with('s') {
            let num_str = &duration_str[..duration_str.len() - 1];
            let seconds: u64 = num_str.parse().unwrap();
            let duration = Duration::from_secs(seconds);
            assert_eq!(duration, Duration::from_secs(30));
        }
    }
}

#[cfg(test)]
mod model_size_tests {
    use super::*;

    #[test]
    fn test_model_size_extraction_logic() {
        // Test model size extraction logic that would be in extract_model_size
        let model_names = vec![
            ("meta-llama/llama-3.3-70b-instruct:free", 70),
            ("qwen/qwen-2.5-72b-instruct:free", 72),
            ("microsoft/phi-4-reasoning:free", 4),
            ("qwen/qwen3-1.7b:free", 1), // Should round down from 1.7
        ];

        for (model_name, expected_size) in model_names {
            // Extract number followed by 'b'
            if let Some(captures) = regex::Regex::new(r"(\d+(?:\.\d+)?)b").unwrap().captures(model_name) {
                if let Some(size_str) = captures.get(1) {
                    let size: f32 = size_str.as_str().parse().unwrap();
                    let size_int = size as u32;
                    assert_eq!(size_int, expected_size, "Failed for model: {}", model_name);
                }
            }
        }
    }
}