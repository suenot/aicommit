//! Common test utilities and helpers for aicommit

use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test utilities for creating temporary git repositories and test environments
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub repo_path: String,
}

impl TestEnvironment {
    /// Creates a new temporary test environment with an initialized git repository
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().to_string_lossy().to_string();

        // Initialize git repository
        std::process::Command::new("git")
            .arg("init")
            .current_dir(&repo_path)
            .output()?;

        // Configure git user for testing
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()?;

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()?;

        Ok(Self { temp_dir, repo_path })
    }

    /// Creates a test file with given content in the repository
    pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = self.temp_dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, content)?;
        Ok(())
    }

    /// Stages all changes in the repository
    pub fn stage_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&self.repo_path)
            .output()?;
        Ok(())
    }

    /// Gets the path to the repository
    pub fn path(&self) -> &str {
        &self.repo_path
    }
}

/// Mock HTTP server utilities for testing API interactions
pub mod mock_server {
    use httpmock::{MockServer, Mock};
    use serde_json::json;

    /// Creates a mock OpenAI-compatible API server
    pub fn create_openai_mock(server: &MockServer) -> Mock {
        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions")
                .header("content-type", "application/json");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "choices": [{
                        "message": {
                            "content": "feat: add new feature for testing"
                        }
                    }],
                    "usage": {
                        "prompt_tokens": 50,
                        "completion_tokens": 10,
                        "total_tokens": 60
                    }
                }));
        })
    }

    /// Creates a mock server that returns an error response
    pub fn create_error_mock(server: &MockServer, status_code: u16) -> Mock {
        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(status_code)
                .json_body(json!({
                    "error": {
                        "message": "API Error for testing",
                        "type": "test_error"
                    }
                }));
        })
    }
}

/// Test data generation utilities
pub mod test_data {
    /// Generates sample git diff output for testing
    pub fn sample_git_diff() -> &'static str {
        r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,6 +1,8 @@
 fn main() {
     println!("Hello, world!");
+    // Added new functionality
+    let result = process_data();
+    println!("Result: {}", result);
 }
"#
    }

    /// Generates sample config for testing
    pub fn sample_config() -> serde_json::Value {
        serde_json::json!({
            "provider": "openai",
            "api_key": "test-api-key",
            "model": "gpt-3.5-turbo",
            "base_url": "https://api.openai.com",
            "max_tokens": 150
        })
    }
}

/// Assertion helpers for common test patterns
pub mod assertions {
    /// Asserts that a commit message follows conventional commit format
    pub fn assert_conventional_commit(message: &str) {
        let conventional_regex = regex::Regex::new(r"^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .+").unwrap();
        assert!(
            conventional_regex.is_match(message),
            "Commit message '{}' does not follow conventional commit format",
            message
        );
    }

    /// Asserts that a commit message is not empty and has reasonable length
    pub fn assert_valid_commit_message(message: &str) {
        assert!(!message.trim().is_empty(), "Commit message should not be empty");
        assert!(message.len() >= 3, "Commit message should be at least 3 characters long");
        assert!(message.len() <= 72, "Commit message should not exceed 72 characters for the first line");
    }
}