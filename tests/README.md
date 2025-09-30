# Testing Infrastructure for AICommit

This directory contains the comprehensive testing infrastructure for the AICommit project, implementing the requirements from Issue #8 (Task 1.4: Set Up Testing Infrastructure).

## 📁 Directory Structure

```
tests/
├── README.md                 # This file - testing documentation
├── common/
│   └── mod.rs                # Common test utilities and helpers
├── unit/
│   └── test_core_utils.rs    # Unit tests for core utility functions
├── integration/
│   └── test_cli.rs           # Integration tests for CLI functionality
└── fixtures/
    ├── sample_diffs.txt      # Sample git diff data for testing
    ├── sample_configs.json   # Sample configuration files
    └── api_responses.json    # Mock API response data
```

## 🧪 Testing Framework Components

### 1. Unit Test Framework

- **Location**: `tests/unit/`
- **Purpose**: Tests individual functions and modules in isolation
- **Framework**: Standard Rust `#[test]` with additional utilities
- **Coverage**: Core utility functions like version parsing, git diff processing, model size extraction

**Key Features:**
- Tests for version increment logic
- Git diff processing validation
- Duration parsing functionality
- Model size extraction from names
- Safe string slicing utilities

### 2. Integration Test Framework

- **Location**: `tests/integration/`
- **Purpose**: End-to-end testing of CLI functionality
- **Framework**: `assert_cmd` for command-line testing
- **Coverage**: CLI argument parsing, git repository interactions, API integrations

**Key Features:**
- CLI help and version testing
- Git repository validation
- Error handling for invalid scenarios
- API integration with mock servers
- Configuration testing

### 3. Test Utilities and Mocks

- **Location**: `tests/common/mod.rs`
- **Purpose**: Shared utilities for all tests
- **Features**:
  - `TestEnvironment`: Creates temporary git repositories
  - `mock_server`: HTTP mock utilities for API testing
  - `test_data`: Sample data generators
  - `assertions`: Common assertion helpers

**Utilities Available:**
```rust
// Create test git repository
let env = TestEnvironment::new()?;
env.create_file("test.rs", "content")?;
env.stage_all()?;

// Mock API servers
let mock = mock_server::create_openai_mock(&server);

// Sample data
let diff = test_data::sample_git_diff();
let config = test_data::sample_config();

// Assertions
assertions::assert_conventional_commit(message);
assertions::assert_valid_commit_message(message);
```

### 4. Test Data and Fixtures

- **Location**: `tests/fixtures/`
- **Purpose**: Static test data for consistent testing
- **Contents**:
  - Sample git diffs for various scenarios
  - Configuration files for different providers
  - Mock API responses for testing error handling

### 5. Test Coverage Reporting

- **Tool**: `cargo-tarpaulin`
- **Configuration**: `tarpaulin.toml` in project root
- **Output**: HTML, LCOV, and JSON formats
- **Location**: Reports generated in `target/coverage/`

**Coverage Features:**
- Source-only coverage (excludes test files)
- Multiple output formats for CI integration
- Configurable failure thresholds
- Detailed uncovered code reporting

## 🚀 Running Tests

### Basic Test Commands

```bash
# Run all tests
npm run test
# or
cargo test

# Run only unit tests
npm run test:unit
# or
cargo test --lib

# Run only integration tests
npm run test:integration
# or
cargo test --test '*'

# Run tests with coverage
npm run test:coverage
# or
cargo tarpaulin --config tarpaulin.toml

# Watch tests during development
npm run test:watch
# or
cargo watch -x test
```

### Test Environment Setup

For integration tests that require external dependencies:

1. **Git Repository**: Tests automatically create temporary git repositories
2. **Mock Servers**: HTTP mocks are created for API testing
3. **Environment Variables**: Set test-specific API keys and configurations

```bash
# Example test environment
export OPENAI_API_KEY="test-key-for-testing"
export RUST_LOG=debug
```

## 📊 Test Categories

### Unit Tests
- ✅ Version parsing and increment logic
- ✅ Git diff processing and truncation
- ✅ Duration string parsing (30s, 5m, 2h)
- ✅ Model size extraction from names
- ✅ Safe string slicing for UTF-8
- ✅ Commit message validation

### Integration Tests
- ✅ CLI help and version commands
- ✅ Git repository detection
- ✅ Staged changes validation
- ✅ Dry-run functionality
- ✅ API error handling
- ✅ Configuration management

### Mock Testing
- ✅ OpenAI API responses
- ✅ Error response handling
- ✅ Rate limiting scenarios
- ✅ Empty/invalid responses

## 🔧 Configuration

### Cargo.toml Dependencies

```toml
[dev-dependencies]
tokio-test = "0.4"        # Async testing utilities
mockall = "0.12"          # Mock object framework
tempfile = "3.8"          # Temporary file management
assert_cmd = "2.0"        # CLI testing framework
predicates = "3.0"        # Assertion predicates
httpmock = "0.7"          # HTTP mock server
insta = "1.0"             # Snapshot testing
```

### Tarpaulin Configuration

- Output formats: HTML, LCOV, JSON
- Timeout: 120 seconds
- Excludes: test files and target directory
- Includes: only source files
- Workspace support enabled

## 🏗️ Architecture Decisions

### Library vs Binary Structure

The project is configured as both a library and binary:
- **Binary**: `src/main.rs` - CLI application
- **Library**: `src/lib.rs` - Testable functions

This allows unit testing of individual functions while maintaining the CLI interface.

### Test Organization

- **Common utilities**: Shared across all test types
- **Fixtures**: Static test data for consistency
- **Separation**: Unit vs integration test clear boundaries
- **Mocking**: External dependencies mocked for reliability

### Future Extensibility

The infrastructure is designed to support:
- Additional AI providers
- New CLI commands and options
- Performance benchmarking
- Property-based testing
- Mutation testing

## 📝 Contributing to Tests

### Adding New Tests

1. **Unit Tests**: Add to appropriate file in `tests/unit/`
2. **Integration Tests**: Add to `tests/integration/test_cli.rs`
3. **Test Data**: Add fixtures to `tests/fixtures/`
4. **Utilities**: Extend `tests/common/mod.rs`

### Test Naming Conventions

```rust
#[test]
fn test_function_name_scenario() {
    // Test implementation
}
```

### Mock Data Guidelines

- Use realistic but safe test data
- Include edge cases and error scenarios
- Document expected behavior in comments
- Keep fixtures minimal but comprehensive

## 🎯 Success Metrics

The testing infrastructure successfully addresses all requirements from Issue #8:

- ✅ **Unit test framework**: Comprehensive unit testing with utilities
- ✅ **Test utilities and mocks**: HTTP mocks and test environment helpers
- ✅ **Integration foundations**: CLI and API integration testing
- ✅ **Test data and fixtures**: Sample data for consistent testing
- ✅ **Coverage reporting**: Tarpaulin configuration with multiple formats

This infrastructure supports test-driven development throughout the refactoring process and provides a solid foundation for maintaining code quality.