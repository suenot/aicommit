//! AICommit library crate
//!
//! This library provides core functionality for generating AI-powered git commit messages.
//! The main binary is located in main.rs, while this lib.rs exposes testable functions.

use regex::Regex;
use std::time::Duration;

/// Increments a semantic version string
///
/// # Arguments
/// * `version` - A version string in the format "major.minor.patch"
///
/// # Returns
/// * `Ok(String)` - The incremented version
/// * `Err(Box<dyn std::error::Error>)` - If the version format is invalid
///
/// # Examples
/// ```
/// use aicommit::increment_version;
///
/// let result = increment_version("1.2.3").unwrap();
/// assert_eq!(result, "1.2.4");
/// ```
pub fn increment_version(version: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid version format. Expected major.minor.patch".into());
    }

    let major: u32 = parts[0].parse()?;
    let minor: u32 = parts[1].parse()?;
    let patch: u32 = parts[2].parse()?;

    Ok(format!("{}.{}.{}", major, minor, patch + 1))
}

/// Processes git diff output to make it more concise for AI processing
///
/// # Arguments
/// * `diff` - Raw git diff output
///
/// # Returns
/// * `String` - Processed diff output
///
/// # Examples
/// ```
/// use aicommit::process_git_diff_output;
///
/// let diff = "diff --git a/test.rs b/test.rs\n+added line\n-removed line";
/// let result = process_git_diff_output(diff);
/// assert!(result.contains("test.rs"));
/// ```
pub fn process_git_diff_output(diff: &str) -> String {
    const MAX_DIFF_CHARS: usize = 15000;
    const MAX_FILE_DIFF_CHARS: usize = 3000;

    if diff.len() <= MAX_DIFF_CHARS {
        return diff.to_string();
    }

    let mut result = String::new();
    let mut current_length = 0;

    for line in diff.lines() {
        if current_length + line.len() > MAX_DIFF_CHARS {
            result.push_str("\n... (diff truncated due to size) ...");
            break;
        }

        result.push_str(line);
        result.push('\n');
        current_length += line.len() + 1;

        // Truncate individual file diffs if they're too long
        if line.starts_with("diff --git") && result.len() > MAX_FILE_DIFF_CHARS {
            result.push_str("... (file diff truncated) ...\n");
        }
    }

    result
}

/// Gets a safe slice length that doesn't break UTF-8 boundaries
///
/// # Arguments
/// * `s` - The string to slice
/// * `max_len` - Maximum desired length
///
/// # Returns
/// * `usize` - Safe length for slicing
///
/// # Examples
/// ```
/// use aicommit::get_safe_slice_length;
///
/// let result = get_safe_slice_length("hello world", 7);
/// assert!(result <= 7);
/// ```
pub fn get_safe_slice_length(s: &str, max_len: usize) -> usize {
    if s.len() <= max_len {
        return s.len();
    }

    // Find the last safe boundary (space or punctuation) within the limit
    for (i, _) in s.char_indices().rev() {
        if i <= max_len {
            if i == 0 || s.chars().nth(i - 1).map_or(false, |c| c.is_whitespace() || c.is_ascii_punctuation()) {
                return i;
            }
        }
    }

    // If no safe boundary found, return max_len but ensure it's a valid char boundary
    s.char_indices()
        .take_while(|(i, _)| *i < max_len)
        .last()
        .map(|(i, _)| i + 1)
        .unwrap_or(max_len.min(s.len()))
}

/// Parses a duration string (e.g., "30s", "5m") into a Duration
///
/// # Arguments
/// * `duration_str` - String representation of duration
///
/// # Returns
/// * `Ok(Duration)` - Parsed duration
/// * `Err(String)` - Error message if parsing fails
///
/// # Examples
/// ```
/// use aicommit::parse_duration;
/// use std::time::Duration;
///
/// let result = parse_duration("30s").unwrap();
/// assert_eq!(result, Duration::from_secs(30));
/// ```
pub fn parse_duration(duration_str: &str) -> Result<Duration, String> {
    if duration_str.ends_with('s') {
        let num_str = &duration_str[..duration_str.len() - 1];
        let seconds: u64 = num_str.parse()
            .map_err(|_| format!("Invalid seconds value: {}", num_str))?;
        Ok(Duration::from_secs(seconds))
    } else if duration_str.ends_with('m') {
        let num_str = &duration_str[..duration_str.len() - 1];
        let minutes: u64 = num_str.parse()
            .map_err(|_| format!("Invalid minutes value: {}", num_str))?;
        Ok(Duration::from_secs(minutes * 60))
    } else if duration_str.ends_with('h') {
        let num_str = &duration_str[..duration_str.len() - 1];
        let hours: u64 = num_str.parse()
            .map_err(|_| format!("Invalid hours value: {}", num_str))?;
        Ok(Duration::from_secs(hours * 3600))
    } else {
        Err(format!("Invalid duration format: {}. Use format like '30s', '5m', '2h'", duration_str))
    }
}

/// Extracts model size from model name string
///
/// # Arguments
/// * `model_name` - Name of the AI model
///
/// # Returns
/// * `u32` - Estimated model size in billions of parameters
///
/// # Examples
/// ```
/// use aicommit::extract_model_size;
///
/// let size = extract_model_size("meta-llama/llama-3.3-70b-instruct:free");
/// assert_eq!(size, 70);
/// ```
pub fn extract_model_size(model_name: &str) -> u32 {
    // Try to find a pattern like "70b" or "3.5b" in the model name
    let re = Regex::new(r"(\d+(?:\.\d+)?)b").unwrap();

    if let Some(captures) = re.captures(model_name) {
        if let Some(size_str) = captures.get(1) {
            if let Ok(size) = size_str.as_str().parse::<f32>() {
                return size as u32;
            }
        }
    }

    // Fallback: if no size found, assume it's a small model
    1
}

/// Validates that a commit message follows basic quality standards
///
/// # Arguments
/// * `message` - The commit message to validate
///
/// # Returns
/// * `Ok(())` - If the message is valid
/// * `Err(String)` - Error description if invalid
///
/// # Examples
/// ```
/// use aicommit::validate_commit_message;
///
/// assert!(validate_commit_message("feat: add new feature").is_ok());
/// assert!(validate_commit_message("").is_err());
/// ```
pub fn validate_commit_message(message: &str) -> Result<(), String> {
    let trimmed = message.trim();

    if trimmed.is_empty() {
        return Err("Commit message cannot be empty".to_string());
    }

    if trimmed.len() < 3 {
        return Err("Commit message must be at least 3 characters long".to_string());
    }

    // First line should not be too long (conventional limit is 50-72 chars)
    if let Some(first_line) = trimmed.lines().next() {
        if first_line.len() > 72 {
            return Err("First line of commit message should not exceed 72 characters".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_version() {
        assert_eq!(increment_version("1.2.3").unwrap(), "1.2.4");
        assert_eq!(increment_version("0.0.0").unwrap(), "0.0.1");
        assert_eq!(increment_version("10.20.30").unwrap(), "10.20.31");

        assert!(increment_version("invalid").is_err());
        assert!(increment_version("1.2").is_err());
        assert!(increment_version("1.2.3.4").is_err());
    }

    #[test]
    fn test_process_git_diff_output() {
        let diff = "diff --git a/test.rs b/test.rs\n+added line\n-removed line";
        let result = process_git_diff_output(diff);
        assert!(result.contains("test.rs"));
        assert!(result.contains("+added line"));
        assert!(result.contains("-removed line"));
    }

    #[test]
    fn test_get_safe_slice_length() {
        assert_eq!(get_safe_slice_length("hello", 10), 5);
        assert_eq!(get_safe_slice_length("hello world", 7), 5); // Should stop at space
        assert!(get_safe_slice_length("hello world", 7) <= 7);
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));

        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("30x").is_err());
    }

    #[test]
    fn test_extract_model_size() {
        assert_eq!(extract_model_size("meta-llama/llama-3.3-70b-instruct:free"), 70);
        assert_eq!(extract_model_size("qwen/qwen-2.5-72b-instruct:free"), 72);
        assert_eq!(extract_model_size("microsoft/phi-4-reasoning:free"), 4);
        assert_eq!(extract_model_size("qwen/qwen3-1.7b:free"), 1);
        assert_eq!(extract_model_size("unknown-model"), 1); // Fallback
    }

    #[test]
    fn test_validate_commit_message() {
        assert!(validate_commit_message("feat: add new feature").is_ok());
        assert!(validate_commit_message("fix: resolve bug").is_ok());

        assert!(validate_commit_message("").is_err());
        assert!(validate_commit_message("  ").is_err());
        assert!(validate_commit_message("ok").is_err()); // Too short

        // Test very long message
        let long_message = "a".repeat(100);
        assert!(validate_commit_message(&long_message).is_err());
    }
}