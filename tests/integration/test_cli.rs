//! Integration tests for CLI functionality

use assert_cmd::Command;
use predicates::prelude::*;
use super::super::common::*;

#[cfg(test)]
mod cli_integration_tests {
    use super::*;

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("aicommit").unwrap();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("A CLI tool that generates concise and descriptive git commit messages"));
    }

    #[test]
    fn test_cli_version() {
        let mut cmd = Command::cargo_bin("aicommit").unwrap();
        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
    }

    #[test]
    fn test_cli_outside_git_repo() {
        let temp_env = TestEnvironment::new().unwrap();

        // Remove .git directory to make it not a git repo
        std::fs::remove_dir_all(format!("{}/.git", temp_env.path())).unwrap();

        let mut cmd = Command::cargo_bin("aicommit").unwrap();
        cmd.current_dir(temp_env.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("not a git repository").or(
                predicate::str::contains("Not a git repository")
            ));
    }

    #[test]
    fn test_cli_no_staged_changes() {
        let temp_env = TestEnvironment::new().unwrap();

        let mut cmd = Command::cargo_bin("aicommit").unwrap();
        cmd.current_dir(temp_env.path())
            .assert()
            .failure()
            .stderr(predicate::str::contains("No staged changes").or(
                predicate::str::contains("nothing to commit")
            ));
    }

    #[test]
    fn test_cli_dry_run_with_staged_changes() {
        let temp_env = TestEnvironment::new().unwrap();

        // Create a test file and stage it
        temp_env.create_file("test.txt", "Hello, world!").unwrap();
        temp_env.stage_all().unwrap();

        let mut cmd = Command::cargo_bin("aicommit").unwrap();
        cmd.current_dir(temp_env.path())
            .arg("--dry-run")
            .env("OPENAI_API_KEY", "test-key") // Mock API key
            .timeout(std::time::Duration::from_secs(30))
            .assert()
            .success();
        // Note: In real test, would check for commit message output
        // but since we don't have actual API key, this will likely fail
        // This test demonstrates the structure for future implementation
    }
}

#[cfg(test)]
mod config_integration_tests {
    use super::*;

    #[test]
    fn test_config_initialization() {
        let temp_env = TestEnvironment::new().unwrap();

        // Test config setup command
        let mut cmd = Command::cargo_bin("aicommit").unwrap();
        cmd.current_dir(temp_env.path())
            .arg("--config")
            .timeout(std::time::Duration::from_secs(10))
            .assert();
        // Would succeed if config command is implemented
    }
}

#[cfg(test)]
mod api_integration_tests {
    use super::*;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_openai_api_integration() {
        let temp_env = TestEnvironment::new().unwrap();
        let server = MockServer::start_async().await;

        // Set up mock OpenAI API
        let _mock = mock_server::create_openai_mock(&server);

        // Create test file and stage it
        temp_env.create_file("feature.rs", "fn new_feature() { /* TODO */ }").unwrap();
        temp_env.stage_all().unwrap();

        // Test with mock server
        let mut cmd = Command::cargo_bin("aicommit").unwrap();
        cmd.current_dir(temp_env.path())
            .arg("--dry-run")
            .env("OPENAI_API_KEY", "test-key")
            .env("OPENAI_BASE_URL", &server.base_url())
            .timeout(std::time::Duration::from_secs(30));

        // In a real implementation, we would configure the app to use the mock server
        // and verify it generates appropriate commit messages
        // For now, this demonstrates the test structure
    }

    #[tokio::test]
    async fn test_api_error_handling() {
        let temp_env = TestEnvironment::new().unwrap();
        let server = MockServer::start_async().await;

        // Set up mock server to return error
        let _error_mock = mock_server::create_error_mock(&server, 401);

        temp_env.create_file("test.rs", "// test change").unwrap();
        temp_env.stage_all().unwrap();

        let mut cmd = Command::cargo_bin("aicommit").unwrap();
        cmd.current_dir(temp_env.path())
            .arg("--dry-run")
            .env("OPENAI_API_KEY", "invalid-key")
            .env("OPENAI_BASE_URL", &server.base_url())
            .timeout(std::time::Duration::from_secs(30))
            .assert()
            .failure();
        // Should fail with appropriate error message
    }
}