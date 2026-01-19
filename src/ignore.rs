// .aicommitignore support module
// Provides parsing and filtering for files that should be excluded from LLM diff analysis

use std::fs;
use std::path::Path;
use glob::Pattern;
use tracing::debug;

/// Default patterns for files that should always be ignored
/// These include binary files, lock files, and other files that don't provide
/// meaningful context for commit message generation
pub const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    // Binary files
    "*.exe",
    "*.dll",
    "*.so",
    "*.dylib",
    "*.bin",
    "*.obj",
    "*.o",
    "*.a",
    "*.lib",
    "*.pyc",
    "*.pyo",
    "*.class",
    "*.jar",
    "*.war",
    "*.ear",
    "*.wasm",

    // Images
    "*.png",
    "*.jpg",
    "*.jpeg",
    "*.gif",
    "*.bmp",
    "*.ico",
    "*.webp",
    "*.svg",
    "*.tiff",
    "*.tif",
    "*.psd",
    "*.ai",
    "*.eps",

    // Audio/Video
    "*.mp3",
    "*.mp4",
    "*.avi",
    "*.mov",
    "*.wmv",
    "*.flv",
    "*.wav",
    "*.ogg",
    "*.webm",
    "*.mkv",

    // Archives
    "*.zip",
    "*.tar",
    "*.gz",
    "*.rar",
    "*.7z",
    "*.bz2",
    "*.xz",
    "*.tar.gz",
    "*.tgz",

    // Lock files
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "Gemfile.lock",
    "poetry.lock",
    "Pipfile.lock",
    "composer.lock",
    "pubspec.lock",
    "go.sum",
    "flake.lock",
    "*.lock",

    // Database files
    "*.db",
    "*.sqlite",
    "*.sqlite3",
    "*.mdb",

    // Fonts
    "*.ttf",
    "*.otf",
    "*.woff",
    "*.woff2",
    "*.eot",

    // Office documents
    "*.pdf",
    "*.doc",
    "*.docx",
    "*.xls",
    "*.xlsx",
    "*.ppt",
    "*.pptx",

    // Minified files
    "*.min.js",
    "*.min.css",

    // Source maps
    "*.map",
    "*.js.map",
    "*.css.map",

    // Build artifacts and generated files
    "*.pb.go",
    "*.pb.cc",
    "*.pb.h",
    "*_generated.go",
    "*_generated.ts",
    "*.generated.cs",
];

/// Structure representing the aicommitignore configuration
#[derive(Debug)]
pub struct AiCommitIgnore {
    patterns: Vec<Pattern>,
}

impl AiCommitIgnore {
    /// Create a new AiCommitIgnore instance
    /// Loads patterns from .aicommitignore file and includes default patterns
    pub fn new() -> Self {
        let mut patterns = Vec::new();

        // Load default patterns first
        for pattern_str in DEFAULT_IGNORE_PATTERNS {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                patterns.push(pattern);
            }
        }

        // Load patterns from .aicommitignore file if it exists
        if let Ok(content) = fs::read_to_string(".aicommitignore") {
            for line in content.lines() {
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Handle negation patterns (lines starting with !)
                // For simplicity, we just skip negation patterns for now
                // A full implementation would require more complex logic
                if line.starts_with('!') {
                    continue;
                }

                if let Ok(pattern) = Pattern::new(line) {
                    patterns.push(pattern);
                } else {
                    debug!("Invalid pattern in .aicommitignore: {}", line);
                }
            }
        }

        Self { patterns }
    }

    /// Check if a file path should be ignored based on the patterns
    pub fn is_ignored(&self, file_path: &str) -> bool {
        // Extract just the filename for pattern matching
        let filename = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);

        for pattern in &self.patterns {
            // Try matching against the full path
            if pattern.matches(file_path) {
                return true;
            }
            // Also try matching against just the filename
            if pattern.matches(filename) {
                return true;
            }
            // Try matching with path components for patterns like "dir/*.ext"
            if pattern.matches_path(Path::new(file_path)) {
                return true;
            }
        }

        false
    }
}

impl Default for AiCommitIgnore {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract file path from a git diff section header
/// Example: "diff --git a/src/main.rs b/src/main.rs" -> "src/main.rs"
pub fn extract_file_path_from_diff_header(header: &str) -> Option<String> {
    // The header format is: "diff --git a/<path> b/<path>"
    // We want to extract the path after "b/"
    if let Some(b_pos) = header.find(" b/") {
        let path_start = b_pos + 3; // Skip " b/"
        let path = header[path_start..].trim();
        // Handle paths with spaces by taking until end of line
        if !path.is_empty() {
            return Some(path.to_string());
        }
    }

    // Alternative: try to extract from "a/" if "b/" fails
    if let Some(a_pos) = header.find("a/") {
        let path_start = a_pos + 2;
        if let Some(b_pos) = header[path_start..].find(" b/") {
            let path = &header[path_start..path_start + b_pos];
            if !path.is_empty() {
                return Some(path.to_string());
            }
        }
    }

    None
}

/// Filter a git diff output by removing sections for ignored files
/// If `skip_filter` is true, returns the original diff unchanged
pub fn filter_diff_by_ignore_patterns(diff: &str, skip_filter: bool) -> String {
    if skip_filter {
        return diff.to_string();
    }
    let ignore = AiCommitIgnore::new();

    // Split the diff into file sections
    let file_pattern = "diff --git ";
    let sections: Vec<&str> = diff.split(file_pattern).collect();

    let mut filtered = String::new();

    // First section might be empty or contain only whitespace
    if !sections.is_empty() && !sections[0].trim().is_empty() {
        filtered.push_str(sections[0]);
    }

    // Process each file section
    for section in sections.iter().skip(1) {
        if section.trim().is_empty() {
            continue;
        }

        // Get the first line which contains the file paths
        let first_line = section.lines().next().unwrap_or("");

        // Extract file path and check if it should be ignored
        if let Some(file_path) = extract_file_path_from_diff_header(&format!("diff --git {}", first_line)) {
            if ignore.is_ignored(&file_path) {
                debug!("Ignoring diff for file: {}", file_path);
                continue;
            }
        }

        // Include this section in the filtered output
        filtered.push_str(file_pattern);
        filtered.push_str(section);
    }

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_path() {
        assert_eq!(
            extract_file_path_from_diff_header("diff --git a/src/main.rs b/src/main.rs"),
            Some("src/main.rs".to_string())
        );
        assert_eq!(
            extract_file_path_from_diff_header("diff --git a/package-lock.json b/package-lock.json"),
            Some("package-lock.json".to_string())
        );
    }

    #[test]
    fn test_default_patterns() {
        let ignore = AiCommitIgnore::new();

        // Test binary files
        assert!(ignore.is_ignored("test.exe"));
        assert!(ignore.is_ignored("lib.dll"));
        assert!(ignore.is_ignored("lib.so"));

        // Test lock files
        assert!(ignore.is_ignored("package-lock.json"));
        assert!(ignore.is_ignored("yarn.lock"));
        assert!(ignore.is_ignored("Cargo.lock"));

        // Test images
        assert!(ignore.is_ignored("image.png"));
        assert!(ignore.is_ignored("photo.jpg"));

        // Test source code should NOT be ignored
        assert!(!ignore.is_ignored("main.rs"));
        assert!(!ignore.is_ignored("index.ts"));
        assert!(!ignore.is_ignored("app.py"));
    }

    #[test]
    fn test_filter_diff() {
        let diff = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
+// New comment
 fn main() {
     println!("Hello");
 }
diff --git a/package-lock.json b/package-lock.json
index aaaaaaa..bbbbbbb 100644
--- a/package-lock.json
+++ b/package-lock.json
@@ -1,100 +1,200 @@
 {
   "name": "test",
   "lockfileVersion": 3
 }
diff --git a/README.md b/README.md
index ccccccc..ddddddd 100644
--- a/README.md
+++ b/README.md
@@ -1 +1,2 @@
 # Project
+New line
"#;

        let filtered = filter_diff_by_ignore_patterns(diff, false);

        // Should contain main.rs diff
        assert!(filtered.contains("src/main.rs"));

        // Should NOT contain package-lock.json diff
        assert!(!filtered.contains("package-lock.json"));

        // Should contain README.md diff
        assert!(filtered.contains("README.md"));

        // Test with skip_filter = true - should contain all files
        let unfiltered = filter_diff_by_ignore_patterns(diff, true);
        assert!(unfiltered.contains("src/main.rs"));
        assert!(unfiltered.contains("package-lock.json"));
        assert!(unfiltered.contains("README.md"));
    }
}
