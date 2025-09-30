use std::fs;
use std::path::Path;
use tracing::{info, warn, error};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};
use tracing_appender::{non_blocking, rolling};
use anyhow::Result;

/// Logging configuration structure
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level filter (trace, debug, info, warn, error)
    pub level: String,
    /// Enable console logging
    pub console_enabled: bool,
    /// Enable file logging
    pub file_enabled: bool,
    /// Log file directory
    pub log_dir: String,
    /// Log file name prefix
    pub file_prefix: String,
    /// Enable structured JSON logging for files
    pub json_format: bool,
    /// Show source code locations
    pub show_target: bool,
    /// Show thread IDs
    pub show_thread_ids: bool,
    /// Enable ANSI colors for console output
    pub ansi_colors: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            console_enabled: true,
            file_enabled: true,
            log_dir: get_default_log_dir(),
            file_prefix: "aicommit".to_string(),
            json_format: false,
            show_target: false,
            show_thread_ids: false,
            ansi_colors: true,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration with environment variable overrides
    pub fn new() -> Self {
        let mut config = Self::default();

        // Override with environment variables if present
        if let Ok(level) = std::env::var("AICOMMIT_LOG_LEVEL") {
            config.level = level;
        }

        if let Ok(log_dir) = std::env::var("AICOMMIT_LOG_DIR") {
            config.log_dir = log_dir;
        }

        if let Ok(_) = std::env::var("AICOMMIT_LOG_JSON") {
            config.json_format = true;
        }

        if let Ok(_) = std::env::var("AICOMMIT_LOG_NO_COLOR") {
            config.ansi_colors = false;
        }

        if let Ok(_) = std::env::var("AICOMMIT_LOG_VERBOSE") {
            config.show_target = true;
            config.show_thread_ids = true;
        }

        config
    }

    /// Enable debug logging with enhanced output
    pub fn with_debug(&mut self) -> &mut Self {
        self.level = "debug".to_string();
        self.show_target = true;
        self
    }

    /// Enable trace logging with full verbosity
    pub fn with_trace(&mut self) -> &mut Self {
        self.level = "trace".to_string();
        self.show_target = true;
        self.show_thread_ids = true;
        self
    }

    /// Disable console logging (file only)
    pub fn file_only(&mut self) -> &mut Self {
        self.console_enabled = false;
        self
    }

    /// Disable file logging (console only)
    pub fn console_only(&mut self) -> &mut Self {
        self.file_enabled = false;
        self
    }
}

/// Get the default log directory based on the platform
fn get_default_log_dir() -> String {
    if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("aicommit").join("logs").to_string_lossy().to_string()
    } else {
        "./logs".to_string()
    }
}

/// Initialize the logging system with the given configuration
/// Returns an optional WorkerGuard that must be kept alive for file logging to work properly
pub fn init_logging(config: &LoggingConfig) -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    // Create log directory if it doesn't exist
    if config.file_enabled {
        ensure_log_dir(&config.log_dir)?;
    }

    // Create the environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let mut layers = Vec::new();
    let mut guard = None;

    // Console layer
    if config.console_enabled {
        let console_layer = fmt::layer()
            .with_ansi(config.ansi_colors)
            .with_target(config.show_target)
            .with_thread_ids(config.show_thread_ids)
            .with_span_events(if config.level == "trace" {
                FmtSpan::FULL
            } else {
                FmtSpan::NONE
            })
            .with_filter(env_filter.clone());

        layers.push(console_layer.boxed());
    }

    // File layer with rotation
    if config.file_enabled {
        let file_appender = rolling::daily(&config.log_dir, &config.file_prefix);
        let (non_blocking_appender, worker_guard) = non_blocking(file_appender);

        let file_layer = if config.json_format {
            fmt::layer()
                .json()
                .with_writer(non_blocking_appender)
                .with_target(true)
                .with_thread_ids(config.show_thread_ids)
                .with_span_events(FmtSpan::CLOSE)
                .with_filter(env_filter.clone())
                .boxed()
        } else {
            fmt::layer()
                .with_writer(non_blocking_appender)
                .with_ansi(false)  // No ANSI colors in files
                .with_target(config.show_target)
                .with_thread_ids(config.show_thread_ids)
                .with_span_events(if config.level == "trace" {
                    FmtSpan::FULL
                } else {
                    FmtSpan::CLOSE
                })
                .with_filter(env_filter)
                .boxed()
        };

        layers.push(file_layer);

        // Store the guard to return it to the caller
        guard = Some(worker_guard);
    }

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(layers)
        .init();

    info!("Logging system initialized");
    info!("Log level: {}", config.level);
    if config.file_enabled {
        info!("Log directory: {}", config.log_dir);
    }

    Ok(guard)
}

/// Ensure the log directory exists
fn ensure_log_dir(log_dir: &str) -> Result<()> {
    let path = Path::new(log_dir);
    if !path.exists() {
        fs::create_dir_all(path)?;
        info!("Created log directory: {}", log_dir);
    }
    Ok(())
}

/// Create a span for tracing operations with context
#[macro_export]
macro_rules! operation_span {
    ($name:expr) => {
        tracing::info_span!("operation", name = $name, id = %uuid::Uuid::new_v4())
    };
    ($name:expr, $($key:ident = $value:expr),*) => {
        tracing::info_span!("operation", name = $name, id = %uuid::Uuid::new_v4(), $($key = $value),*)
    };
}

/// Log an error with context and return it
pub fn log_error<E>(error: E, context: &str) -> E
where
    E: std::fmt::Display + std::fmt::Debug,
{
    error!("{}: {}", context, error);
    error
}

/// Log a warning with context
pub fn log_warning(message: &str, context: &str) {
    warn!("{}: {}", context, message);
}

/// Log an info message with context
pub fn log_info(message: &str, context: &str) {
    info!("{}: {}", context, message);
}

/// Initialize logging with default configuration
pub fn init_default_logging() -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let config = LoggingConfig::new();
    init_logging(&config)
}

/// Initialize logging for development (debug level, verbose output)
pub fn init_dev_logging() -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let mut config = LoggingConfig::new();
    config.with_debug();
    init_logging(&config)
}

/// Initialize logging for production (info level, structured)
pub fn init_prod_logging() -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let mut config = LoggingConfig::new();
    config.json_format = true;
    config.ansi_colors = false;
    init_logging(&config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logging_config_creation() {
        let config = LoggingConfig::new();
        assert_eq!(config.level, "info");
        assert!(config.console_enabled);
        assert!(config.file_enabled);
    }

    #[test]
    fn test_logging_config_debug() {
        let mut config = LoggingConfig::new();
        config.with_debug();
        assert_eq!(config.level, "debug");
        assert!(config.show_target);
    }

    #[test]
    fn test_log_dir_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("test_logs").to_string_lossy().to_string();

        ensure_log_dir(&log_dir).unwrap();
        assert!(Path::new(&log_dir).exists());
    }
}