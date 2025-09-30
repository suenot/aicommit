use aicommit::logging::{LoggingConfig, init_logging};
use tracing::{info, warn, error, debug, trace};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test basic logging configuration
    let mut config = LoggingConfig::new();
    config.with_debug();

    // Initialize logging
    init_logging(&config)?;

    // Test different log levels
    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");

    // Test structured logging
    info!(user_id = 123, operation = "test", "User performed test operation");

    println!("Logging test completed successfully!");
    Ok(())
}